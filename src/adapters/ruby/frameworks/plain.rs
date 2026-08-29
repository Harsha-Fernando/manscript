use crate::adapters::ruby::run_bundle;
use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::utils::filesystem::write_file;

pub struct PlainRubyFramework;

impl FrameworkAdapter for PlainRubyFramework {
    fn id(&self) -> &'static str {
        "ruby"
    }

    fn language(&self) -> &'static str {
        "ruby"
    }

    fn default_language_version(&self) -> &'static str {
        "3.4"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("ruby app.rb".into()),
            test: None,
            build: None,
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("app.rb"),
            "puts \"Hello from ManScript (Ruby, no framework).\"\n",
        )?;
        write_file(
            &ctx.project_root.join("Gemfile"),
            "source \"https://rubygems.org\"\n",
        )?;
        let project = Project {
            root: ctx.project_root.to_path_buf(),
            config: crate::adapters::traits::default_project_config(
                ctx.project_name,
                "ruby",
                self.default_language_version(),
                None,
                "bundler",
                self.default_commands(ctx.project_name),
            ),
        };
        let _ = run_bundle(&project, &["install".into()]);
        Ok(())
    }
}
