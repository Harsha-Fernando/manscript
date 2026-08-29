# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-08-29

### Added

- `manscript completions` prints a Tab-completion script for bash, zsh, fish, PowerShell, and Elvish
- After the script is loaded, Tab completes commands and `create` stack ids (`django`, `python`, `c`, …)

## [0.1.0] - 2026-08-29

### Added

- Language-agnostic CLI (`manscript`) with clap
- `manscript.toml` configuration and project discovery
- Adapter registry for languages and frameworks
- Python adapter with venv isolation (`.manscript/environment`)
- Ruby adapter with Bundler path isolation
- Frameworks: Django, FastAPI, Flask, Rails, Sinatra
- Language-only create: `manscript create python` / `create ruby` / `create c` / `create cpp` / `create java`, or interactive **None** (no `[framework]` in toml)
- Django in-project `create`: apps (urls + hello view) and models (admin + migrate)
- Runtime providers: system, uv (Python), mise (Ruby)
- Commands: create, init, setup, install, run, test, build, doctor, env
- Dual license: MIT OR Apache-2.0
