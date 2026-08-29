# ManScript

ManScript 0.1.4 is a language-agnostic CLI for creating and running projects in isolated development environments. It keeps project tools under `.manscript/environment`, so you do not need to activate a virtual environment or permanently change your shell.

Supported languages are Python, Ruby, C, C++, Java, Go, Rust, PHP, and C#. Supported frameworks remain Django, FastAPI, Flask, Rails, and Sinatra; framework work is maintenance- and demand-driven while language environments are the primary direction.

## Install

ManScript 0.1.4 is installed from source and requires [Rust](https://rustup.rs):

```bash
git clone https://github.com/Harsha-Fernando/manscript.git
cd manscript
cargo install --path .
manscript doctor
```

Cargo normally installs the binary in `~/.cargo/bin` (Windows: `%USERPROFILE%\.cargo\bin`). See the [installation guide](docs/install.html) if `manscript` is not found.

## Choose your starting point

### Start a new project

```bash
manscript create
cd myproject
manscript run
```

The wizard asks for a language, an optional framework, and a project name. C, C++, Java, Go, Rust, PHP, and C# have only one project kind, so they skip the framework prompt.

### Adopt an existing, unconfigured project

```bash
cd existing-project
manscript init
manscript setup
manscript run
```

Review the generated `manscript.toml` before setup, especially its versions and commands.

### Use a cloned ManScript project

If the repository already contains `manscript.toml`:

```bash
cd cloned-project
manscript setup
manscript run
```

By default, `setup` first uses a suitable system runtime. When Go, Rust, PHP, or .NET is unavailable, ManScript can offer a confirmed mise-managed fallback stored below `MANSCRIPT_HOME`; `--yes` accepts that prompt. An explicit `[runtime].provider` overrides this selection order. Dependency preparation runs `go mod download`, `cargo fetch`, `composer install`, or `dotnet restore` as appropriate. The project environment itself is not expected to be committed.

## Common commands

| Command | Purpose |
|---|---|
| `manscript create` | Create a project, or generate an app/module inside one |
| `manscript init` | Add `manscript.toml` to the current directory |
| `manscript setup` | Prepare the runtime, environment, and dependencies |
| `manscript install` | Install dependencies into an existing project environment |
| `manscript run`, `test`, `build` | Run the corresponding configured command |
| `manscript doctor` | Diagnose local tools without changing anything |
| `manscript env` | Show resolved project and environment paths |
| `manscript shell` | Open a project-aware child shell |
| `manscript completions` | Print completion code for a supported shell |

Read the complete [command reference](docs/commands.html).

## Terminal experience

ManScript uses the same compact structure across commands:

- `INFO` introduces the command and its purpose.
- Numbered steps show progress for multi-stage work such as `create` and `setup`.
- `DONE` confirms successful completion and is followed by relevant next commands.
- `WARN` identifies recoverable states; `ERROR` explains what failed and how to continue.

Interactive prompts use arrow keys, Enter, and Escape without changing terminal fonts or settings. Colors and animation are disabled automatically when the terminal does not support them or when `NO_COLOR` is set.
The root help and version screens keep ManScript's full wordmark as prominent brand moments; individual command output stays compact.

## Development shell

After `manscript setup`, run:

```bash
manscript shell
```

ManScript starts a child shell in the project root with managed tools first on `PATH`. Python uses its virtual-environment tools; Ruby also receives its project Bundler and gem variables; C, C++, and Java use project shims for detected system toolchains. Go, Rust, PHP, and C# use project tool shims plus project-local module, package, build-cache, and CLI-state variables. The parent terminal and shell startup files remain unchanged.

Type `exit` or send EOF to return. See the [shell guide](docs/shell.html).

## Configuration and command safety

`manscript.toml` declares the project name, language and version, optional framework, recorded environment manager, optional runtime provider, and optional `run`, `test`, and `build` commands. In 0.1.4, the language adapter controls the environment implementation.

Configured commands are parsed into a program and arguments. They are not sent through a command shell: `sudo`, path traversal in the program, and shell metacharacters such as `|`, `&&`, `$`, redirects, and backticks are rejected. Treat `manscript.toml` as trusted project configuration, like a Makefile. See [configuration](docs/config.html) and [security](SECURITY.md).

## Isolation and caches

- Project environments live at `.manscript/environment`.
- Downloaded runtime tools live below `~/.manscript/tools` by default.
- Set `MANSCRIPT_HOME` to move that user-level cache.
- Go module/build caches, Rust Cargo state and targets, Composer home, and NuGet/.NET CLI state are kept project-local.
- ManScript never uninstalls or disables your system Python, Ruby, PHP, Go, Rust, .NET SDK, Java, or compilers.

## Documentation

The plain static documentation site is in [`docs/`](docs/). Start with the [overview](docs/index.html), or read about [creating projects](docs/create.html), [troubleshooting](docs/troubleshooting.html), and [uninstalling](docs/uninstall.html).

## Development

When working on ManScript itself, the binary is not installed automatically:

```bash
cargo build
cargo test
cargo run -- --help
```

The `--` separates Cargo arguments from ManScript arguments.

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.
