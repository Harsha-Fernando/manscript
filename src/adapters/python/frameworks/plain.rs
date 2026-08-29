use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainPythonFramework;

impl FrameworkAdapter for PlainPythonFramework {
    fn id(&self) -> &'static str {
        "python"
    }

    fn language(&self) -> &'static str {
        "python"
    }

    fn default_language_version(&self) -> &'static str {
        "3.13"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("python main.py".into()),
            test: None,
            build: None,
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("main.py"),
            r#"def main():
    print("Hello from ManScript (Python, no framework).")


if __name__ == "__main__":
    main()
"#,
        )?;
        write_file(
            &ctx.project_root.join("requirements.txt"),
            "# add packages here\n",
        )?;
        Ok(())
    }
}
