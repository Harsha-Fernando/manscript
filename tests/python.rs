use manscript::core::registry::default_registry;
use manscript::core::runtime::version_matches;
use manscript::process::executor::split_command_line;

#[test]
fn django_run_command_splits() {
    let argv = split_command_line("python manage.py runserver").unwrap();
    assert_eq!(argv, ["python", "manage.py", "runserver"]);
}

#[test]
fn python_versions() {
    assert!(version_matches("3.13.5", "3.13"));
    assert!(version_matches("3.14.0", "3.13"));
    assert!(!version_matches("3.12.8", "3.13"));
}

#[test]
fn python_plain_is_language_only() {
    let r = default_registry();
    assert!(r.framework("python").unwrap().language_only());
    assert!(!r.framework("django").unwrap().language_only());
}

#[test]
fn django_installed_apps_insert() {
    let src = "INSTALLED_APPS = [\n    'django.contrib.admin',\n]\n";
    let out =
        manscript::adapters::python::frameworks::django::insert_installed_app(src, "blog").unwrap();
    assert!(out.contains("'blog',"));
}

#[test]
fn django_url_include_insert() {
    let src =
        "from django.urls import path\nurlpatterns = [\n    path('admin/', admin.site.urls),\n]\n";
    let out =
        manscript::adapters::python::frameworks::django::insert_url_include(src, "blog").unwrap();
    assert!(out.contains("include('blog.urls')"));
}
