# Changelog

All notable changes to this project are documented here.

## [Unreleased]

### UX and documentation

- Reorganized the documentation around three onboarding paths: create a new project, initialize an existing project, or set up a cloned ManScript project
- Added complete command, configuration, troubleshooting, and development-shell guides
- Clarified command mutation and project prerequisites, Python generator naming, single-kind generator prompts, Rails asset builds, `MANSCRIPT_HOME`, and optional uninstall cleanup
- Standardized site navigation, accessibility metadata, skip links, responsive layouts, dark mode, focus states, and reduced-motion behavior
- Replaced clickable code blocks with accessible copy buttons and a Clipboard API fallback
- Improved shell lifecycle messaging: the project prompt is Unix-only, missing environments are errors, and successful exit confirms that the original terminal is unchanged

## [0.1.2] - 2026-08-29

### Added

- `manscript shell` opens a project-aware child shell with managed tools first on `PATH`
- Language adapters expose shell environment variables and path entries for Python, Ruby, C, C++, Java, and future integrations
- Tests cover project and environment detection, path ordering, environment preservation, shell help, and shell configuration isolation

### Security

- The development shell modifies only the child process environment and never writes shell startup files
- Interactive shells are launched directly as argv with inherited terminal streams; no command strings or `sh -c` wrappers are used
- Project names are sanitized before being included in the Unix child prompt

### Fixed

- Java CLI tests reject macOS launcher stubs that exist but cannot start a real Java runtime

## [0.1.1] - 2026-08-29

### Added

- `manscript completions` prints a Tab-completion script for bash, zsh, fish, PowerShell, and Elvish
- Loaded completion scripts complete commands and `create` stack identifiers

## [0.1.0] - 2026-08-29

### Added

- Language-agnostic `manscript` CLI and `manscript.toml` project discovery
- Adapter registry for Python, Ruby, C, C++, Java, Django, FastAPI, Flask, Rails, and Sinatra
- Python virtual environments, Ruby Bundler isolation, and system compiler/JDK project shims
- System, uv, and mise runtime providers
- Commands for create, init, setup, install, run, test, build, doctor, and env
- Dual MIT or Apache-2.0 licensing
