use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainCppFramework;

impl FrameworkAdapter for PlainCppFramework {
    fn id(&self) -> &'static str {
        "cpp"
    }

    fn language(&self) -> &'static str {
        "cpp"
    }

    fn default_language_version(&self) -> &'static str {
        "any"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("app".into()),
            test: None,
            build: Some("c++".into()),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("main.cpp"),
            r#"#include <iostream>

int main() {
    std::cout << "Hello from ManScript (C++, no framework).\n";
    return 0;
}
"#,
        )
    }
}
