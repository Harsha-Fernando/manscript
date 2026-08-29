use crate::adapters::ruby::{run_bundle, RubyAdapter};
use crate::adapters::traits::{
    FrameworkAdapter, GenerateContext, GeneratorSpec, LanguageAdapter, ScaffoldContext,
};
use crate::config::CommandsConfig;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::utils::filesystem::{append_unique, write_file};

pub struct SinatraFramework;

const GENERATORS: &[GeneratorSpec] = &[GeneratorSpec {
    id: "routes",
    label: "Routes",
    hint: "another routes file, required from app.rb",
}];

impl FrameworkAdapter for SinatraFramework {
    fn id(&self) -> &'static str {
        "sinatra"
    }

    fn language(&self) -> &'static str {
        "ruby"
    }

    fn default_language_version(&self) -> &'static str {
        "3.4"
    }

    fn default_framework_version(&self) -> &'static str {
        "4.1"
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("ruby app.rb".into()),
            test: None,
            build: None,
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
                Some(("sinatra", self.default_framework_version())),
                "bundler",
                self.default_commands(ctx.project_name),
            ),
        };

        write_file(
            &ctx.project_root.join("Gemfile"),
            &format!(
                r#"source "https://rubygems.org"

gem "sinatra", "~> {}"
gem "rackup"
gem "puma"
"#,
                self.default_framework_version()
            ),
        )?;
        write_file(
            &ctx.project_root.join("app.rb"),
            r#"require "sinatra"

set :bind, "127.0.0.1"
set :port, 4567

get "/" do
  "Hello from ManScript + Sinatra"
end
"#,
        )?;
        write_file(
            &ctx.project_root.join("config.ru"),
            "require \"./app\"\nrun Sinatra::Application\n",
        )?;

        RubyAdapter.install_packages(&project, &[])?;
        run_bundle(&project, &["install".into()])?;
        Ok(())
    }

    fn generate(&self, ctx: &GenerateContext<'_>, kind: &str, name: &str) -> Result<()> {
        if kind != "routes" {
            return Err(ManscriptError::Message(format!(
                "Sinatra does not support the `{kind}` generator.\n\nUse:\n\n    manscript create routes <name>"
            )));
        }
        let file = ctx.project.root.join("routes").join(format!("{name}.rb"));
        if file.exists() {
            return Err(ManscriptError::Message(format!(
                "Routes `{name}` already exist."
            )));
        }
        write_file(
            &file,
            &format!(
                r#"get "/{name}" do
  "Hello from {name}"
end
"#
            ),
        )?;
        let app_rb = ctx.project.root.join("app.rb");
        if !app_rb.is_file() {
            return Err(ManscriptError::Message(
                "ManScript created the routes file but could not load it because `app.rb` is missing.\n\nRestore the Sinatra entry file, then add the corresponding `require_relative` line."
                    .into(),
            ));
        }
        append_unique(&app_rb, &format!("require_relative \"routes/{name}\"\n"))?;
        ctx.printer
            .info(&format!("Routes `{name}` are required from app.rb."));
        Ok(())
    }
}
