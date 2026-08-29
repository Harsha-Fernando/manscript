use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainCFramework;

impl FrameworkAdapter for PlainCFramework {
    fn id(&self) -> &'static str {
        "c"
    }

    fn language(&self) -> &'static str {
        "c"
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
            build: Some("cc".into()),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("main.c"),
            r#"#include <stdio.h>

int main(void) {
    printf("Hello from ManScript (C, no framework).\n");
    return 0;
}
"#,
        )
    }
}
