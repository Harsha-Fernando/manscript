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

ManScript never requires `source .venv/bin/activate`. It invokes binaries from the project environment directly.

## Does ManScript turn off Python or Django?

No. Your system `python`, `django-admin`, `ruby`, and `rails` stay exactly as they were.

ManScript does not uninstall them or remove them from PATH. It also does not “activate” a virtualenv in your shell.

- `python` in the terminal → whatever your shell already had (Homebrew, pyenv, …)
- `manscript run` → this project’s isolated interpreter and packages

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

Use `--yes` / `-y` for non-interactive confirmation (CI).

## How it works

```
CLI → core (project, config, registry)
        → language adapters (Python, Ruby, C, C++, Java)
        → framework adapters (Django, FastAPI, Flask, Rails, Sinatra)
        → runtime providers (system, uv for Python, mise for Ruby)
```

The core does not know about pip, uv, Bundler, or mise. Providers are replaceable. If a suitable system runtime exists, ManScript uses it. Otherwise it can download an isolated runtime into `~/.manscript` after confirmation (never sudo).

Project isolation lives in `.manscript/environment` inside the project.

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

ManScript runs subprocesses. It will not escalate privileges, silently modify system files, overwrite a non-empty project without confirmation, or send your project anywhere.

See [SECURITY.md](SECURITY.md).

## License

MIT. See [LICENSE](LICENSE).
