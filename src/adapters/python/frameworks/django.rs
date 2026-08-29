use crate::adapters::traits::{
    FrameworkAdapter, GenerateContext, GeneratorSpec, LanguageAdapter, ScaffoldContext,
};
use crate::config::CommandsConfig;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::process::{Executor, PreparedCommand};
use crate::utils::filesystem::write_file;
use crate::utils::platform::python_bin_name;
use crate::utils::prompts::{select, Choice};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct DjangoFramework;

const GENERATORS: &[GeneratorSpec] = &[
    GeneratorSpec {
        id: "app",
        label: "App",
        hint: "startapp, INSTALLED_APPS, urls, a hello view",
    },
    GeneratorSpec {
        id: "model",
        label: "Model",
        hint: "model + admin + makemigrations + migrate",
    },
];

impl FrameworkAdapter for DjangoFramework {
    fn id(&self) -> &'static str {
        "django"
    }

    fn language(&self) -> &'static str {
        "python"
    }

    fn default_language_version(&self) -> &'static str {
        "3.13"
    }

    fn default_framework_version(&self) -> &'static str {
        "5.2"
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("python manage.py runserver".into()),
            test: Some("python manage.py test".into()),
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
        let python = ctx.environment.bin_dir.join(python_bin_name());
        let pin = format!("Django=={}", self.default_framework_version());
        crate::adapters::python::PythonAdapter.install_packages(
            &Project {
                root: ctx.project_root.to_path_buf(),
                config: crate::adapters::traits::default_project_config(
                    ctx.project_name,
                    "python",
                    self.default_language_version(),
                    Some(("django", self.default_framework_version())),
                    "venv",
                    self.default_commands(ctx.project_name),
                ),
            },
            std::slice::from_ref(&pin),
        )?;

        let prepared = PreparedCommand {
            program: python,
            args: vec![
                "-m".into(),
                "django".into(),
                "startproject".into(),
                ctx.project_name.to_string(),
                ctx.project_root.display().to_string(),
            ],
            cwd: ctx.project_root.to_path_buf(),
            extra_env: HashMap::new(),
            path_prepend: vec![ctx.environment.bin_dir.clone()],
        };
        Executor::new().run_status(prepared)?;

        write_file(
            &ctx.project_root.join("requirements.txt"),
            &format!("{pin}\n"),
        )?;
        write_file(&ctx.project_root.join("requirements").join(".keep"), "")?;
        Ok(())
    }

    fn generate(&self, ctx: &GenerateContext<'_>, kind: &str, name: &str) -> Result<()> {
        match kind {
            "app" => generate_app(ctx, name),
            "model" => generate_model(ctx, name),
            other => Err(ManscriptError::Message(format!(
                "Django does not support the `{other}` generator through ManScript.\n\nAvailable generator types:\n  app, model"
            ))),
        }
    }
}

fn python_bin(ctx: &GenerateContext<'_>) -> Result<PathBuf> {
    let python = ctx.environment.bin_dir.join(python_bin_name());
    if !python.is_file() {
        return Err(ManscriptError::EnvironmentNotReady(
            ctx.project.environment_dir(),
        ));
    }
    Ok(python)
}

fn manage_py(ctx: &GenerateContext<'_>, python: &Path, args: &[&str]) -> Result<()> {
    let mut argv = vec!["manage.py".into()];
    argv.extend(args.iter().map(|s| (*s).to_string()));
    let prepared = PreparedCommand {
        program: python.to_path_buf(),
        args: argv,
        cwd: ctx.project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: vec![ctx.environment.bin_dir.clone()],
    };
    Executor::new().run_status(prepared)
}

fn generate_app(ctx: &GenerateContext<'_>, name: &str) -> Result<()> {
    let python = python_bin(ctx)?;
    let dest = ctx.project.root.join(name);
    if dest.exists() {
        return Err(ManscriptError::Message(format!(
            "An app named `{name}` already exists in this project."
        )));
    }
    manage_py(ctx, &python, &["startapp", name])?;

    let settings = find_settings_py(&ctx.project.root).ok_or_else(|| {
        ManscriptError::Message(
            "Django created the app, but ManScript could not find `settings.py` to update `INSTALLED_APPS`.\n\nAdd the app to `INSTALLED_APPS` manually, or restore the project settings file."
                .into(),
        )
    })?;
    let text = std::fs::read_to_string(&settings)?;
    std::fs::write(&settings, insert_installed_app(&text, name)?)?;
    ctx.printer
        .info(&format!("App `{name}` is in INSTALLED_APPS."));

    write_file(
        &dest.join("views.py"),
        &format!(
            r#"from django.http import HttpResponse


def hello(request):
    return HttpResponse("Hello from {name}")
"#
        ),
    )?;
    write_file(
        &dest.join("urls.py"),
        r#"from django.urls import path

from . import views

urlpatterns = [
    path("hello/", views.hello),
]
"#,
    )?;

    let project_urls = settings
        .parent()
        .map(|p| p.join("urls.py"))
        .filter(|p| p.is_file());
    if let Some(urls) = project_urls {
        let body = std::fs::read_to_string(&urls)?;
        std::fs::write(&urls, insert_url_include(&body, name)?)?;
        ctx.printer
            .info(&format!("Routed `/{name}/hello/` in the project urls."));
    } else {
        ctx.printer
            .warn("The app was created, but the project `urls.py` file was not found. Add the app routes manually.");
    }
    Ok(())
}

fn generate_model(ctx: &GenerateContext<'_>, name: &str) -> Result<()> {
    let python = python_bin(ctx)?;
    let class_name = python_class_name(name)?;
    let app = pick_django_app(ctx)?;
    let app_dir = ctx.project.root.join(&app);
    if !app_dir.join("apps.py").is_file() {
        return Err(ManscriptError::Message(format!(
            "`{app}` does not appear to be a Django app because `{app}/apps.py` is missing.\n\nChoose an existing Django app or restore its `apps.py` file."
        )));
    }

    let models_py = app_dir.join("models.py");
    let mut models = if models_py.is_file() {
        std::fs::read_to_string(&models_py)?
    } else {
        "from django.db import models\n".into()
    };
    if !models.contains("from django.db import models") {
        models = format!("from django.db import models\n{models}");
    }
    if models.contains(&format!("class {class_name}(")) {
        return Err(ManscriptError::Message(format!(
            "`{class_name}` already exists in {app}/models.py."
        )));
    }
    if !models.ends_with('\n') {
        models.push('\n');
    }
    models.push_str(&format!(
        r#"

class {class_name}(models.Model):
    title = models.CharField(max_length=200)

    def __str__(self):
        return self.title
"#
    ));
    write_file(&models_py, &models)?;

    let admin_py = app_dir.join("admin.py");
    let mut admin = if admin_py.is_file() {
        std::fs::read_to_string(&admin_py)?
    } else {
        "from django.contrib import admin\n".into()
    };
    let import_line = format!("from .models import {class_name}");
    if !admin.contains(&import_line) {
        if !admin.ends_with('\n') {
            admin.push('\n');
        }
        admin.push_str(&import_line);
        admin.push('\n');
    }
    let register = format!("admin.site.register({class_name})");
    if !admin.contains(&register) {
        admin.push_str(&register);
        admin.push('\n');
    }
    write_file(&admin_py, &admin)?;

    manage_py(ctx, &python, &["makemigrations", &app])?;
    manage_py(ctx, &python, &["migrate"])?;
    ctx.printer.info(&format!(
        "Model `{class_name}` in `{app}` is registered in admin and migrated."
    ));
    Ok(())
}

fn pick_django_app(ctx: &GenerateContext<'_>) -> Result<String> {
    let apps = list_django_apps(&ctx.project.root);
    if apps.is_empty() {
        return Err(ManscriptError::Message(
            "No Django apps found (directories with apps.py). Create an app first: manscript create blog"
                .into(),
        ));
    }
    if apps.len() == 1 {
        return Ok(apps[0].clone());
    }
    if ctx.yes {
        return Err(ManscriptError::Message(format!(
            "This project has several apps ({}). Run without -y and pick one, or create the model interactively.",
            apps.join(", ")
        )));
    }
    let owned: Vec<(String, String)> = apps.iter().map(|a| (a.clone(), a.clone())).collect();
    let choices: Vec<Choice<'_>> = owned
        .iter()
        .map(|(id, label)| Choice {
            id: id.as_str(),
            label: label.as_str(),
            hint: "",
        })
        .collect();
    select("Which app should own this model?", &choices, 0)
}

fn list_django_apps(root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut apps: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_dir() {
                return None;
            }
            if !path.join("apps.py").is_file() {
                return None;
            }
            e.file_name().to_str().map(str::to_string)
        })
        .filter(|n| n != "django" && !n.starts_with('.'))
        .collect();
    apps.sort();
    apps
}

pub fn python_class_name(name: &str) -> Result<String> {
    if name.is_empty()
        || !name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ManscriptError::InvalidProjectName(name.to_string()));
    }
    let class: String = name
        .split('_')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_ascii_lowercase(),
            }
        })
        .collect();
    if class.is_empty() {
        return Err(ManscriptError::InvalidProjectName(name.to_string()));
    }
    Ok(class)
}

pub fn insert_installed_app(settings: &str, app: &str) -> Result<String> {
    let quoted_s = format!("'{app}'");
    let quoted_d = format!("\"{app}\"");
    if settings.contains(&quoted_s) || settings.contains(&quoted_d) {
        return Ok(settings.to_string());
    }
    let marker = "INSTALLED_APPS";
    let start = settings.find(marker).ok_or_else(|| {
        ManscriptError::Message(
            "`settings.py` does not contain an `INSTALLED_APPS` assignment.\n\nAdd the app to `INSTALLED_APPS` manually."
                .into(),
        )
    })?;
    let after = &settings[start..];
    let open_rel = after.find('[').ok_or_else(|| {
        ManscriptError::Message(
            "`INSTALLED_APPS` in `settings.py` is not written as a list or tuple that ManScript can update.\n\nAdd the app manually."
                .into(),
        )
    })?;
    let open = start + open_rel;
    let close = matching_bracket(settings, open).ok_or_else(|| {
        ManscriptError::Message(
            "ManScript could not find the end of `INSTALLED_APPS` in `settings.py`.\n\nCheck the settings syntax and add the app manually."
                .into(),
        )
    })?;
    let indent = "    ";
    let insertion = format!("{indent}{quoted_s},\n");
    Ok(format!(
        "{}{}{}",
        &settings[..close],
        insertion,
        &settings[close..]
    ))
}

pub fn insert_url_include(urls: &str, app: &str) -> Result<String> {
    let needle = format!("include('{app}.urls')");
    let needle_d = format!("include(\"{app}.urls\")");
    let mut body = urls.to_string();
    if body.contains(&needle) || body.contains(&needle_d) {
        return Ok(body);
    }
    body = ensure_include_import(&body);
    let marker = "urlpatterns";
    let start = body.find(marker).ok_or_else(|| {
        ManscriptError::Message(
            "The project `urls.py` file does not contain a `urlpatterns` assignment.\n\nInclude the app routes manually."
                .into(),
        )
    })?;
    let after = &body[start..];
    let open_rel = after
        .find('[')
        .ok_or_else(|| {
            ManscriptError::Message(
                "`urlpatterns` in `urls.py` is not a list that ManScript can update.\n\nInclude the app routes manually."
                    .into(),
            )
        })?;
    let open = start + open_rel;
    let close = matching_bracket(&body, open)
        .ok_or_else(|| {
            ManscriptError::Message(
                "ManScript could not find the end of `urlpatterns` in `urls.py`.\n\nCheck the URL configuration syntax and include the app routes manually."
                    .into(),
            )
        })?;
    let insertion = format!("    path('{app}/', include('{app}.urls')),\n");
    Ok(format!("{}{}{}", &body[..close], insertion, &body[close..]))
}

fn ensure_include_import(urls: &str) -> String {
    if urls
        .lines()
        .any(|l| l.contains("from django.urls import") && l.contains("include"))
    {
        return urls.to_string();
    }
    if let Some(line) = urls
        .lines()
        .find(|l| l.trim_start().starts_with("from django.urls import "))
    {
        if line.contains("include") {
            return urls.to_string();
        }
        let updated = if line.trim_end().ends_with("path") {
            line.replacen("import path", "import include, path", 1)
        } else {
            format!("{line}, include")
        };
        return urls.replacen(line, &updated, 1);
    }
    format!("from django.urls import include, path\n{urls}")
}

fn matching_bracket(s: &str, open: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.get(open) != Some(&b'[') {
        return None;
    }
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_settings_py(root: &Path) -> Option<PathBuf> {
    fn walk(dir: &Path, depth: usize) -> Option<PathBuf> {
        if depth > 4 {
            return None;
        }
        let skip = [
            ".manscript",
            ".git",
            "__pycache__",
            "site-packages",
            "node_modules",
            "vendor",
            "environment",
        ];
        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
            if skip.contains(&name) {
                return None;
            }
        }
        let candidate = dir.join("settings.py");
        if candidate.is_file() {
            return Some(candidate);
        }
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = walk(&path, depth + 1) {
                    return Some(found);
                }
            }
        }
        None
    }
    walk(root, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_into_installed_apps() {
        let src = r#"
INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
]
"#;
        let out = insert_installed_app(src, "blog").unwrap();
        assert!(out.contains("'blog',"));
        let again = insert_installed_app(&out, "blog").unwrap();
        assert_eq!(
            out.matches("'blog'").count(),
            again.matches("'blog'").count()
        );
    }

    #[test]
    fn inserts_url_include_once() {
        let src = r#"
from django.contrib import admin
from django.urls import path

urlpatterns = [
    path('admin/', admin.site.urls),
]
"#;
        let out = insert_url_include(src, "blog").unwrap();
        assert!(out.contains("include"));
        assert!(out.contains("include('blog.urls')"));
        let again = insert_url_include(&out, "blog").unwrap();
        assert_eq!(
            out.matches("include('blog.urls')").count(),
            again.matches("include('blog.urls')").count()
        );
    }

    #[test]
    fn python_class_name_rejects_junk() {
        assert!(python_class_name("../x").is_err());
        assert!(python_class_name("post").unwrap() == "Post");
        assert!(python_class_name("blog_post").unwrap() == "BlogPost");
        assert!(python_class_name("Post").unwrap() == "Post");
    }
}
