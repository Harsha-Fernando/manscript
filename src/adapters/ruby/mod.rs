use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, EnvironmentKind, ShellEnvironment};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::{split_command_line, Executor, PreparedCommand};
use crate::utils::filesystem::{ensure_dir, write_file};
use crate::utils::platform::{env_bin_dir, exe_name};

pub mod frameworks;
pub mod package_manager;
pub mod runtime;

pub struct RubyAdapter;

impl LanguageAdapter for RubyAdapter {
    fn id(&self) -> &'static str {
        "ruby"
    }

    fn package_manager_name(&self) -> &'static str {
        "bundler"
    }

    fn default_environment_manager(&self) -> &'static str {
        "bundler"
    }

    fn create_environment(&self, project: &Project, runtime: &Runtime) -> Result<Environment> {
        let root = project.environment_dir();
        ensure_dir(&root)?;
        ensure_dir(&env_bin_dir(&root))?;

        let bundle_dir = project.root.join(".bundle");
        ensure_dir(&bundle_dir)?;
        write_file(
            &bundle_dir.join("config"),
            "---\nBUNDLE_PATH: \".manscript/environment\"\nBUNDLE_BIN: \".manscript/environment/bin\"\nBUNDLE_DISABLE_SHARED_GEMS: \"true\"\n",
        )?;

        ensure_bundler(project, runtime)?;

        Ok(Environment {
            bin_dir: env_bin_dir(&root),
            root,
            kind: EnvironmentKind::BundlerPath,
        })
    }

    fn environment_ready(&self, project: &Project) -> bool {
        project.environment_dir().is_dir() && project.root.join(".bundle").join("config").is_file()
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        if !project.root.join("Gemfile").is_file() {
            return Ok(());
        }
        let ruby = ruby_from_env_or_path(project)?;
        bundle(project, &ruby, &["install".into()])
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        let ruby = ruby_from_env_or_path(project)?;
        let gem = gem_command(&ruby);
        for pkg in packages {
            let mut args = vec![
                "install".into(),
                pkg.clone(),
                "--install-dir".into(),
                project.environment_dir().display().to_string(),
                "--bindir".into(),
                project.environment_bin_dir().display().to_string(),
                "--no-document".into(),
            ];
            if let Some((name, ver)) = pkg.split_once(':') {
                args = vec![
                    "install".into(),
                    name.to_string(),
                    "-v".into(),
                    ver.to_string(),
                    "--install-dir".into(),
                    project.environment_dir().display().to_string(),
                    "--bindir".into(),
                    project.environment_bin_dir().display().to_string(),
                    "--no-document".into(),
                ];
            }
            let prepared = PreparedCommand {
                program: gem.clone(),
                args,
                cwd: project.root.clone(),
                extra_env: ruby_env(project, &ruby),
                path_prepend: vec![project.environment_bin_dir(), gem_bindir(&ruby)],
            };
            Executor::new().run_status(prepared)?;
        }
        Ok(())
    }

    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand> {
        if !self.environment_ready(project) {
            return Err(ManscriptError::EnvironmentNotReady(
                project.environment_dir(),
            ));
        }
        let mut argv = split_command_line(command)?;
        argv.extend(extra_args.iter().cloned());
        let program_name = argv.remove(0);
        let ruby = ruby_from_env_or_path(project)?;
        let program = resolve_ruby_program(project, &ruby, &program_name)?;
        Ok(PreparedCommand {
            program,
            args: argv,
            cwd: project.root.clone(),
            extra_env: ruby_env(project, &ruby),
            path_prepend: vec![project.environment_bin_dir(), gem_bindir(&ruby)],
        })
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        let ruby = ruby_from_env_or_path(project)?;
        Ok(ShellEnvironment {
            path_prepend: vec![project.environment_bin_dir(), gem_bindir(&ruby)],
            extra_env: ruby_env(project, &ruby),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();
        match which::which("ruby") {
            Ok(p) => {
                let ver = crate::runtime::system::probe_version(&p, &["--version"])
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| p.display().to_string());
                checks.push(DoctorCheck {
                    group: "Ruby".into(),
                    ok: true,
                    label: format!("Ruby {ver} detected"),
                    recommendation: None,
                });
            }
            Err(_) => checks.push(DoctorCheck {
                group: "Ruby".into(),
                ok: false,
                label: "Ruby not found".into(),
                recommendation: Some(
                    "ManScript can prepare an isolated Ruby runtime.\n\nRun:\n\n    manscript setup".into(),
                ),
            }),
        }

        let bundler = which::which("bundle").is_ok() || which::which("bundler").is_ok();
        checks.push(if bundler {
            DoctorCheck {
                group: "Ruby package manager".into(),
                ok: true,
                label: "Bundler detected".into(),
                recommendation: None,
            }
        } else {
            DoctorCheck {
                group: "Ruby package manager".into(),
                ok: false,
                label: "Bundler not detected".into(),
                recommendation: Some(
                    "ManScript will install Bundler into the project environment during setup."
                        .into(),
                ),
            }
        });
        checks
    }
}

fn ensure_bundler(project: &Project, runtime: &Runtime) -> Result<()> {
    if project
        .environment_bin_dir()
        .join(exe_name("bundle"))
        .is_file()
        || which::which("bundle").is_ok()
    {
        return Ok(());
    }
    let gem = gem_command(&runtime.executable);
    let prepared = PreparedCommand {
        program: gem,
        args: vec![
            "install".into(),
            "bundler".into(),
            "--install-dir".into(),
            project.environment_dir().display().to_string(),
            "--bindir".into(),
            project.environment_bin_dir().display().to_string(),
            "--no-document".into(),
        ],
        cwd: project.root.clone(),
        extra_env: ruby_env(project, &runtime.executable),
        path_prepend: vec![
            project.environment_bin_dir(),
            gem_bindir(&runtime.executable),
        ],
    };
    Executor::new().run_status(prepared)
}

pub fn ruby_env(project: &Project, ruby: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();
    let gem_home = project.environment_dir().display().to_string();
    env.insert("GEM_HOME".into(), gem_home.clone());
    env.insert("GEM_PATH".into(), gem_home);
    env.insert(
        "BUNDLE_PATH".into(),
        project.environment_dir().display().to_string(),
    );
    env.insert(
        "BUNDLE_BIN".into(),
        project.environment_bin_dir().display().to_string(),
    );
    env.insert(
        "BUNDLE_GEMFILE".into(),
        project.root.join("Gemfile").display().to_string(),
    );
    if let Some(prefix) = ruby.parent().and_then(|p| p.parent()) {
        env.insert("RUBY_ROOT".into(), prefix.display().to_string());
    }
    env
}

fn gem_bindir(ruby: &Path) -> PathBuf {
    ruby.parent().unwrap_or(Path::new(".")).to_path_buf()
}

fn gem_command(ruby: &Path) -> PathBuf {
    let sibling = gem_bindir(ruby).join(exe_name("gem"));
    if sibling.is_file() {
        sibling
    } else {
        which::which(exe_name("gem")).unwrap_or(sibling)
    }
}

fn ruby_from_env_or_path(project: &Project) -> Result<PathBuf> {
    let in_env = project.environment_bin_dir().join(exe_name("ruby"));
    if in_env.is_file() {
        return Ok(in_env);
    }
    which::which(exe_name("ruby")).map_err(|_| ManscriptError::RuntimeNotFound {
        language: "ruby".into(),
        version: project.language_version().into(),
    })
}

fn resolve_ruby_program(project: &Project, ruby: &Path, name: &str) -> Result<PathBuf> {
    if name == "ruby" {
        return Ok(ruby.to_path_buf());
    }
    if name == "bundle" || name == "bundler" {
        let in_env = project.environment_bin_dir().join(exe_name("bundle"));
        if in_env.is_file() {
            return Ok(in_env);
        }
        if let Ok(p) = which::which(exe_name("bundle")) {
            return Ok(p);
        }
    }
    let in_env = project.environment_bin_dir().join(exe_name(name));
    if in_env.is_file() {
        return Ok(in_env);
    }
    let in_project = project.root.join(name);
    if in_project.is_file() {
        return Ok(in_project);
    }
    let bin_rails = project.root.join("bin").join(name);
    if bin_rails.is_file() {
        return Ok(bin_rails);
    }
    Err(ManscriptError::InvalidCommand(format!(
        "could not find `{name}` in the Ruby environment or project bin directory; run `manscript setup` and check the configured command"
    )))
}

fn bundle(project: &Project, ruby: &Path, args: &[String]) -> Result<()> {
    let bundle_bin = resolve_ruby_program(project, ruby, "bundle")?;
    let prepared = PreparedCommand {
        program: bundle_bin,
        args: args.to_vec(),
        cwd: project.root.clone(),
        extra_env: ruby_env(project, ruby),
        path_prepend: vec![project.environment_bin_dir(), gem_bindir(ruby)],
    };
    Executor::new().run_status(prepared)
}

pub fn run_bundle(project: &Project, args: &[String]) -> Result<()> {
    let ruby = ruby_from_env_or_path(project)?;
    bundle(project, &ruby, args)
}
