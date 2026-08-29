# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.1.4] - 2026-08-30

### Added

- Added language-only project support for Go 1.25, Rust stable (edition 2024), PHP 8.4, and C#/.NET 10, including starter files and suitable run, test, and build commands
- Added system-runtime-first resolution with a confirmed mise-managed fallback under `MANSCRIPT_HOME`
- Added project-local dependency and cache environments for Go modules, Cargo, Composer, NuGet, and .NET CLI state
- Added dependency setup through direct `go mod download`, `cargo fetch`, `composer install`, and `dotnet restore` argv execution

### Changed

- Introduced a compact, consistent command presentation with purpose lines, aligned project context, completion summaries, and relevant next actions
- Kept the full ManScript wordmark on root help and version output while keeping individual command output compact
- Unified `create` and `setup` around numbered progress pipelines and refined the development-shell entry and exit states
- Replaced boxed interactive prompts with a lighter keyboard-driven selection layout
- Standardized status and no-op output across init, install, run, test, build, doctor, env, and completions
- Clarified that language environments are the primary direction and framework work is maintenance- and demand-driven

### Security

- Kept new language setup and dependency operations shell-free and sudo-free, with writes bounded to the project root or configured `MANSCRIPT_HOME`
- Added a confirmed project-local Composer bootstrap that verifies the official installer's SHA-384 signature before execution

### Fixed

- Prevented direct completion commands from flooding interactive terminals with generated code
- Made generated zsh completion code initialize `compinit` when `compdef` is unavailable
- Changed completion guidance to show the command users should evaluate instead of the raw generator invocation

## [0.1.3] - 2026-08-29

### UX and documentation

- Reorganized the documentation around three onboarding paths: create a new project, initialize an existing project, or set up a cloned ManScript project
- Added complete command, configuration, troubleshooting, and development-shell guides
- Clarified command mutation and project prerequisites, Python generator naming, single-kind generator prompts, Rails asset builds, `MANSCRIPT_HOME`, and optional uninstall cleanup
- Standardized site navigation, accessibility metadata, skip links, responsive layouts, dark mode, focus states, and reduced-motion behavior
- Kept command blocks clickable and keyboard-accessible, with clear copy feedback and a Clipboard API fallback
- Simplified root help around new, existing, and cloned project paths; grouped commands by purpose and reduced version output to one line
- Improved missing completion-shell and unknown-command errors without suggesting unrelated commands
- Removed heavy vertical callout accents, replaced the floating back-to-top control, and clarified source-update instructions
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
