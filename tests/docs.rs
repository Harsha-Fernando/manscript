use std::fs;
use std::path::{Path, PathBuf};

const PAGES: &[&str] = &[
    "index.html",
    "install.html",
    "create.html",
    "commands.html",
    "config.html",
    "troubleshooting.html",
    "shell.html",
    "uninstall.html",
];

const COMMANDS: &[&str] = &[
    "create",
    "init",
    "setup",
    "install",
    "run",
    "test",
    "build",
    "doctor",
    "env",
    "shell",
    "completions",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn local_hrefs(html: &str) -> Vec<&str> {
    let mut hrefs = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("href=\"") {
        rest = &rest[start + 6..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let href = &rest[..end];
        if !href.starts_with("http")
            && !href.starts_with('#')
            && !href.starts_with("mailto:")
            && !href.is_empty()
        {
            hrefs.push(href);
        }
        rest = &rest[end + 1..];
    }
    hrefs
}

#[test]
fn documentation_pages_are_accessible_and_links_resolve() {
    let docs = root().join("docs");
    for page in PAGES {
        let path = docs.join(page);
        let html =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert!(
            html.contains("<meta name=\"description\""),
            "{page} needs a description"
        );
        assert!(
            html.contains("class=\"skip-link\""),
            "{page} needs a skip link"
        );
        assert!(
            html.contains("<main id=\"main\""),
            "{page} needs a main landmark target"
        );
        assert!(
            html.contains("aria-current=\"page\""),
            "{page} needs an active navigation item"
        );

        for href in local_hrefs(&html) {
            let target = href.split(['#', '?']).next().unwrap_or(href);
            assert!(
                docs.join(target).exists(),
                "{page} links to missing local target `{href}`"
            );
        }
    }
}

#[test]
fn command_reference_covers_every_cli_command() {
    let commands = fs::read_to_string(root().join("docs/commands.html")).unwrap();
    for command in COMMANDS {
        assert!(
            commands.contains(&format!("manscript {command}")),
            "commands.html is missing `manscript {command}`"
        );
    }
}

#[test]
fn published_version_is_consistent_in_release_docs() {
    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    let version = cargo
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|line| line.strip_suffix('"'))
        .expect("package version");
    let install = fs::read_to_string(root().join("docs/install.html")).unwrap();
    let changelog = fs::read_to_string(root().join("CHANGELOG.md")).unwrap();

    assert!(install.contains(version), "install page version is stale");
    assert!(
        changelog.contains(&format!("## [{version}]")),
        "changelog is missing the package version"
    );
}
