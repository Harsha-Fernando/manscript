use crate::adapters::ruby::{ruby_env, run_bundle, RubyAdapter};
use crate::adapters::traits::{
    FrameworkAdapter, GenerateContext, GeneratorSpec, LanguageAdapter, ScaffoldContext,
};
use crate::config::CommandsConfig;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::utils::filesystem::write_file;
use crate::utils::platform::exe_name;

pub struct RailsFramework;

const GENERATORS: &[GeneratorSpec] = &[
    GeneratorSpec {
        id: "scaffold",
        label: "Scaffold",
        hint: "model, controller, views, routes — the whole set",
    },
    GeneratorSpec {
        id: "resource",
        label: "Resource",
        hint: "controller and routes, no extra opinions",
    },
    GeneratorSpec {
        id: "controller",
        label: "Controller",
        hint: "just a controller",
    },
    GeneratorSpec {
        id: "model",
        label: "Model",
        hint: "just a model",
    },
];

impl FrameworkAdapter for RailsFramework {
    fn id(&self) -> &'static str {
        "rails"
    }

    fn language(&self) -> &'static str {
        "ruby"
    }

    fn default_language_version(&self) -> &'static str {
        "3.4"
    }

    fn default_framework_version(&self) -> &'static str {
        "8.0"
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("bin/rails server".into()),
            test: Some("bin/rails test".into()),
            build: Some("bin/rails assets:precompile".into()),
        }
    }

    fn extra_create_steps(&self) -> usize {
        1
    }

    fn generators(&self) -> &'static [GeneratorSpec] {
        GENERATORS
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        let project = Project {
            root: ctx.project_root.to_path_buf(),
            config: crate::adapters::traits::default_project_config(
                ctx.project_name,
                "ruby",
                self.default_language_version(),
                Some(("rails", self.default_framework_version())),
                "bundler",
                self.default_commands(ctx.project_name),
            ),
        };

        write_file(
            &ctx.project_root.join("Gemfile"),
            &format!(
                r#"source "https://rubygems.org"

ruby "{}"

gem "rails", "~> {}"
"#,
                self.default_language_version(),
                self.default_framework_version()
            ),
        )?;

        RubyAdapter.install_packages(&project, &["rails".into()])?;

        let rails_bin = ctx.environment.bin_dir.join(exe_name("rails"));
        let program = if rails_bin.is_file() {
            rails_bin
        } else {
            ctx.project_root.join("bin").join("rails")
        };

        // rails new into this directory; --force allows existing .manscript / Gemfile
        let prepared = crate::process::PreparedCommand {
            program,
            args: vec![
                "new".into(),
                ".".into(),
                "--force".into(),
                "--skip-git".into(),
                "--skip-bundle".into(),
                "--database=sqlite3".into(),
            ],
            cwd: ctx.project_root.to_path_buf(),
            extra_env: crate::adapters::ruby::ruby_env(&project, &ctx.runtime.executable),
            path_prepend: vec![ctx.environment.bin_dir.clone()],
        };
        crate::process::Executor::new().run_status(prepared)?;
        run_bundle(&project, &["install".into()])?;
        Ok(())
    }

    fn generate(&self, ctx: &GenerateContext<'_>, kind: &str, name: &str) -> Result<()> {
        if !GENERATORS.iter().any(|g| g.id == kind) {
            return Err(ManscriptError::Message(format!(
                "Rails can add scaffold, resource, controller, or model — not `{kind}`."
            )));
        }
        let rails_bin = ctx.environment.bin_dir.join(exe_name("rails"));
        let program = if rails_bin.is_file() {
            rails_bin
        } else {
            let binstub = ctx.project.root.join("bin").join("rails");
            if !binstub.is_file() {
                return Err(ManscriptError::EnvironmentNotReady(
                    ctx.project.environment_dir(),
                ));
            }
            binstub
        };
        let prepared = crate::process::PreparedCommand {
            program,
            args: vec!["generate".into(), kind.to_string(), name.to_string()],
            cwd: ctx.project.root.clone(),
            extra_env: ruby_env(ctx.project, &ctx.runtime.executable),
            path_prepend: vec![ctx.environment.bin_dir.clone()],
        };
        crate::process::Executor::new().run_status(prepared)?;
        ctx.printer.info(&format!(
            "Rails `{kind}` `{name}` generated via the project env."
        ));
        Ok(())
    }
}
