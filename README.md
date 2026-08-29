# ManScript

ManScript is a language-agnostic development environment manager designed to make project setup simple and reproducible.

A developer should be able to go from an empty machine to a running app without manually installing a language, creating a virtual environment, activating it, or remembering framework commands.

**While developing this repo** (`manscript` is not on your PATH until you install it):

```bash
cargo run -- --help
cargo run -- doctor
cargo run -- create django myproject
```

The `--` separates Cargo’s flags from ManScript’s flags.

**Install so you can type `manscript` anywhere:**

```bash
cargo install --path .
# if zsh still says "command not found", add ~/.cargo/bin to PATH:
#   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc

manscript doctor
manscript create django myproject
# language only:
# manscript create python myapp
cd myproject
manscript run
```

**Supported:** macOS, Linux, and Windows via [rustup](https://rustup.rs) and `cargo install`. Homebrew and downloadable binaries are not part of 0.1. Human docs: [`docs/`](docs/) (deploy on Vercel; set the project root directory to `docs`).

Python and Ruby work as **language-only** projects (`manscript create python myapp`) or with frameworks (Django, FastAPI, Flask, Rails, Sinatra). C, C++, and Java are language-only (`manscript create c hello`, `cpp`, `java`). Additional languages can be added later through adapters.

## Why

Typical setup is a pile of one-off commands: install a runtime, fix `PATH`, create a venv, install packages, remember `manage.py` vs `bin/rails`. ManScript encodes those requirements in `manscript.toml` and prepares an isolated environment for you.

ManScript never requires `source .venv/bin/activate`. It can invoke binaries from the project environment directly or open a temporary project-aware child shell.

## Does ManScript turn off Python or Django?

No. Your system `python`, `django-admin`, `ruby`, and `rails` stay exactly as they were.

ManScript does not uninstall them or permanently remove them from PATH. `manscript shell` changes only the environment of the child shell it launches.

- `python` in the terminal → whatever your shell already had (Homebrew, pyenv, …)
- `manscript run` → this project’s isolated interpreter and packages
- `manscript shell` → a child shell where this project’s `python`, `pip`, `django-admin`, `ruby`, and other managed tools are first on `PATH`
- `exit` → returns to the original terminal with its previous `PATH`

See `manscript env` inside a project for the exact paths.

## Install

**macOS, Linux, and Windows** (Rust required):

```bash
git clone https://github.com/Harsha-Fernando/manscript.git
cd manscript
cargo install --path .
```

Add `~/.cargo/bin` to PATH if `manscript` is not found (Windows: `%USERPROFILE%\.cargo\bin`).

Homebrew and downloadable binaries are not part of 0.1. To remove ManScript, see [`docs/uninstall.html`](docs/uninstall.html).

## Commands

| Command | Purpose |
|---|---|
| `manscript create` | Interactive or `manscript create django myproject` |
| `manscript init` | Write `manscript.toml` in an existing directory |
| `manscript setup` | Prepare runtime, environment, and dependencies |
| `manscript install` | Install dependencies into the managed environment |
| `manscript run` / `test` / `build` | Execute configured commands |
| `manscript doctor` | Diagnose the machine (never mutates) |
| `manscript env` | Print resolved environment paths |
| `manscript shell` | Open a child shell with project-managed tools on `PATH` |
| `manscript completions` | Print a Tab-completion script (`bash`, `zsh`, `fish`, `powershell`, `elvish`) |

Use `--yes` / `-y` for non-interactive confirmation (CI).

**Tab completion (zsh):** after `manscript` is on PATH, add this to `~/.zshrc` and open a new terminal:

```bash
eval "$(manscript completions zsh)"
```

Then `manscript` + Tab lists commands (`create`, `run`, `shell`, `doctor`, …). `manscript create` + Tab suggests stacks (`django`, `python`, `c`, …). Completing the word `manscript` itself is the shell completing a binary on PATH (`~/.cargo/bin`).

## How it works

```
CLI → core (project, config, registry)
        → language adapters (Python, Ruby, C, C++, Java)
        → framework adapters (Django, FastAPI, Flask, Rails, Sinatra)
        → runtime providers (system, uv for Python, mise for Ruby)
```

The core does not know about pip, uv, Bundler, or mise. Providers are replaceable. If a suitable system runtime exists, ManScript uses it. Otherwise it can download an isolated runtime into `~/.manscript` after confirmation (never sudo).

Project isolation lives in `.manscript/environment` inside the project.

### Development shell

From a directory containing a `manscript.toml` file, or any child directory:

```bash
manscript shell
which python
python --version
exit
```

On Unix and macOS, ManScript launches `$SHELL` (falling back to `/bin/sh`). On Windows it uses `COMSPEC` (falling back to PowerShell). The project environment bin directory is prepended only to the child process `PATH`; existing entries and environment variables are preserved. Ruby-specific Bundler and gem variables come from the Ruby adapter. ManScript does not edit `.bashrc`, `.zshrc`, profiles, or other global shell configuration.

If the project environment has not been prepared, run `manscript setup`. If no `manscript.toml` is found, run the command from inside a ManScript project.

## Configuration

```toml
name = "myproject"

[language]
name = "python"
version = "3.13"

[framework]
name = "django"
version = "5.2"

[environment]
manager = "venv"

[commands]
run = "python manage.py runserver"
test = "python manage.py test"
```

Commands are executed as argv (no shell). `sudo` and shell metacharacters are rejected.

## Security

ManScript runs subprocesses. It will not escalate privileges, silently modify system files, overwrite a non-empty project without confirmation, or send your project anywhere. Configured project commands are always parsed as argv rather than passed to a command shell. `manscript shell` is an explicit interactive-shell boundary and does not execute a configured command string.

See [SECURITY.md](SECURITY.md).

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.
