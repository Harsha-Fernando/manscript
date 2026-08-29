use crate::adapters::traits::{
    FrameworkAdapter, GenerateContext, GeneratorSpec, LanguageAdapter, ScaffoldContext,
};
use crate::config::CommandsConfig;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::utils::filesystem::{append_unique, write_file};

pub struct FlaskFramework;

const GENERATORS: &[GeneratorSpec] = &[GeneratorSpec {
    id: "blueprint",
    label: "Blueprint",
    hint: "a Flask mini-app, registered for you",
}];

impl FrameworkAdapter for FlaskFramework {
    fn id(&self) -> &'static str {
        "flask"
    }

    fn language(&self) -> &'static str {
        "python"
    }

    fn default_language_version(&self) -> &'static str {
        "3.13"
    }

    fn default_framework_version(&self) -> &'static str {
        "3.1"
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("python -m flask --app app run --debug --host 127.0.0.1 --port 5000".into()),
            test: Some("python -m pytest".into()),
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
        let pin = format!("Flask=={}", self.default_framework_version());
        crate::adapters::python::PythonAdapter.install_packages(
            &Project {
                root: ctx.project_root.to_path_buf(),
                config: crate::adapters::traits::default_project_config(
                    ctx.project_name,
                    "python",
                    self.default_language_version(),
                    Some(("flask", self.default_framework_version())),
                    "venv",
                    self.default_commands(ctx.project_name),
                ),
            },
            &[pin.clone()],
        )?;

        write_file(
            &ctx.project_root.join("app.py"),
            r#"from flask import Flask

app = Flask(__name__)


@app.get("/")
def index():
    return {"message": "Hello from ManScript + Flask"}
"#,
        )?;
        write_file(
            &ctx.project_root.join("requirements.txt"),
            &format!("{pin}\n"),
        )?;
        Ok(())
    }

    fn generate(&self, ctx: &GenerateContext<'_>, kind: &str, name: &str) -> Result<()> {
        if kind != "blueprint" {
            return Err(ManscriptError::Message(format!(
                "Flask can add a `blueprint`, not `{kind}`."
            )));
        }
        let pkg = ctx.project.root.join("blueprints").join(name);
        if pkg.exists() {
            return Err(ManscriptError::Message(format!(
                "A blueprint named `{name}` already exists."
            )));
        }
        write_file(&ctx.project.root.join("blueprints").join("__init__.py"), "")?;
        write_file(
            &pkg.join("__init__.py"),
            &format!(
                r#"from flask import Blueprint

bp = Blueprint("{name}", __name__)


@bp.get("/")
def index():
    return {{"app": "{name}"}}
"#
            ),
        )?;
        let app_py = ctx.project.root.join("app.py");
        if !app_py.is_file() {
            return Err(ManscriptError::Message(
                "Expected app.py in this Flask project so the blueprint can be registered.".into(),
            ));
        }
        append_unique(
            &app_py,
            &format!(
                "from blueprints.{name} import bp as {name}_bp\napp.register_blueprint({name}_bp, url_prefix=\"/{name}\")\n"
            ),
        )?;
        ctx.printer
            .info(&format!("Blueprint `{name}` is registered at /{name}."));
        Ok(())
    }
}
