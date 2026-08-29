use crate::adapters::traits::{
    FrameworkAdapter, GenerateContext, GeneratorSpec, LanguageAdapter, ScaffoldContext,
};
use crate::config::CommandsConfig;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::utils::filesystem::{append_unique, write_file};

pub struct FastApiFramework;

const GENERATORS: &[GeneratorSpec] = &[GeneratorSpec {
    id: "router",
    label: "Router",
    hint: "an APIRouter, included on the app for you",
}];

impl FrameworkAdapter for FastApiFramework {
    fn id(&self) -> &'static str {
        "fastapi"
    }

    fn language(&self) -> &'static str {
        "python"
    }

    fn default_language_version(&self) -> &'static str {
        "3.13"
    }

    fn default_framework_version(&self) -> &'static str {
        "0.115"
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("python -m uvicorn main:app --reload --host 127.0.0.1 --port 8000".into()),
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
        let fastapi = format!("fastapi=={}", self.default_framework_version());
        let packages = vec![fastapi, "uvicorn[standard]".into()];
        crate::adapters::python::PythonAdapter.install_packages(
            &Project {
                root: ctx.project_root.to_path_buf(),
                config: crate::adapters::traits::default_project_config(
                    ctx.project_name,
                    "python",
                    self.default_language_version(),
                    Some(("fastapi", self.default_framework_version())),
                    "venv",
                    self.default_commands(ctx.project_name),
                ),
            },
            &packages,
        )?;

        write_file(
            &ctx.project_root.join("main.py"),
            r#"from fastapi import FastAPI

app = FastAPI()


@app.get("/")
def read_root():
    return {"message": "Hello from ManScript + FastAPI"}
"#,
        )?;
        write_file(
            &ctx.project_root.join("requirements.txt"),
            &format!(
                "fastapi=={}\nuvicorn[standard]\n",
                self.default_framework_version()
            ),
        )?;
        Ok(())
    }

    fn generate(&self, ctx: &GenerateContext<'_>, kind: &str, name: &str) -> Result<()> {
        if kind != "router" {
            return Err(ManscriptError::Message(format!(
                "FastAPI can add a `router`, not `{kind}`."
            )));
        }
        let file = ctx.project.root.join("routers").join(format!("{name}.py"));
        if file.exists() {
            return Err(ManscriptError::Message(format!(
                "A router named `{name}` already exists."
            )));
        }
        write_file(&ctx.project.root.join("routers").join("__init__.py"), "")?;
        write_file(
            &file,
            &format!(
                r#"from fastapi import APIRouter

router = APIRouter(prefix="/{name}", tags=["{name}"])


@router.get("/")
def index():
    return {{"app": "{name}"}}
"#
            ),
        )?;
        let main_py = ctx.project.root.join("main.py");
        if !main_py.is_file() {
            return Err(ManscriptError::Message(
                "Expected main.py in this FastAPI project so the router can be included.".into(),
            ));
        }
        append_unique(
            &main_py,
            &format!(
                "from routers.{name} import router as {name}_router\napp.include_router({name}_router)\n"
            ),
        )?;
        ctx.printer
            .info(&format!("Router `{name}` is included at /{name}."));
        Ok(())
    }
}
