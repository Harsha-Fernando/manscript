use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainCsharpFramework;

impl FrameworkAdapter for PlainCsharpFramework {
    fn id(&self) -> &'static str {
        "csharp"
    }

    fn language(&self) -> &'static str {
        "csharp"
    }

    fn default_language_version(&self) -> &'static str {
        "10.0"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, project_name: &str) -> CommandsConfig {
        let project_file = project_file_name(project_name);
        CommandsConfig {
            run: Some(format!("dotnet run --project {project_file}")),
            test: None,
            build: Some(format!("dotnet build {project_file} --nologo")),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("Program.cs"),
            "Console.WriteLine(\"Hello from ManScript (C#, no framework).\");\n",
        )?;

        let identifier = sanitize_dotnet_identifier(ctx.project_name);
        write_file(
            &ctx.project_root.join(project_file_name(ctx.project_name)),
            &format!(
                r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <AssemblyName>{identifier}</AssemblyName>
    <RootNamespace>{identifier}</RootNamespace>
  </PropertyGroup>
</Project>
"#
            ),
        )
    }
}

pub(crate) fn project_file_name(project_name: &str) -> String {
    format!("{}.csproj", sanitize_dotnet_identifier(project_name))
}

fn sanitize_dotnet_identifier(name: &str) -> String {
    let mut sanitized = String::with_capacity(name.len() + 8);
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            sanitized.push(c);
        } else {
            sanitized.push('_');
        }
    }
    if sanitized.trim_matches('_').is_empty() {
        sanitized.clear();
        sanitized.push_str("ManscriptApp");
    } else if sanitized
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit())
    {
        sanitized.insert_str(0, "App_");
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_file_name_is_safe_and_stable() {
        assert_eq!(project_file_name("hello-world"), "hello_world.csproj");
        assert_eq!(project_file_name("2026-app"), "App_2026_app.csproj");
        assert_eq!(project_file_name("../"), "ManscriptApp.csproj");
    }

    #[test]
    fn default_commands_name_the_sanitized_project() {
        let commands = PlainCsharpFramework.default_commands("hello-world");
        assert_eq!(
            commands.run.as_deref(),
            Some("dotnet run --project hello_world.csproj")
        );
        assert_eq!(
            commands.build.as_deref(),
            Some("dotnet build hello_world.csproj --nologo")
        );
    }
}
