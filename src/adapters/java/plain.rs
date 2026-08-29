use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainJavaFramework;

impl FrameworkAdapter for PlainJavaFramework {
    fn id(&self) -> &'static str {
        "java"
    }

    fn language(&self) -> &'static str {
        "java"
    }

    fn default_language_version(&self) -> &'static str {
        "17"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("java -cp . Main".into()),
            test: None,
            build: Some("javac".into()),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("Main.java"),
            r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello from ManScript (Java, no framework).");
    }
}
"#,
        )
    }
}
