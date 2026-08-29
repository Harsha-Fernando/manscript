# Security Policy

## Supported versions

The 0.1.x line is the current development release.

## Reporting a vulnerability

Do not open a public issue for a security problem. Contact the maintainers privately and include:

- A clear description of the issue
- Steps to reproduce it
- The expected and actual behavior
- The potential impact, such as unexpected process execution, path traversal, or privilege concerns

## Trust model

ManScript executes language runtimes, package managers, compilers, and project commands on the user's behalf. Review a cloned project's `manscript.toml` before running `manscript setup`, `install`, `run`, `test`, or `build`. Treat the file as trusted executable configuration, similar to a Makefile.

ManScript does not:

- Invoke `sudo` or deliberately raise privileges
- Run configured commands through a command shell
- Silently modify system files or shell startup files
- Modify the parent process environment
- Overwrite a non-empty project without confirmation
- Upload project contents to a remote service

## Configured command model

Values under `[commands]` are split into a program and argument vector. Single and double quotes group arguments, and backslashes escape the next character outside single quotes. No variable expansion, globbing, pipes, redirects, command substitution, or command chaining takes place.

ManScript rejects:

- `sudo` as a command token
- `..` path traversal in the program
- Shell metacharacters: `|`, `;`, `&`, `>`, `<`, backticks, and `$`
- Empty commands and unclosed quotes

The first token must also resolve through the selected language adapter: generally a managed environment tool or a project-local executable. Additional arguments supplied after `manscript run`, `test`, or `build` are appended as literal argv values.

This reduces shell-injection risk; it does not make an untrusted executable or dependency safe.

## Interactive shell boundary

`manscript shell` deliberately launches an interactive shell executable. It does not execute a configured command and does not pass `-c` or another command string.

On Unix and macOS, the executable comes from `SHELL`, with `/bin/sh` as the fallback. On Windows, it comes from `COMSPEC`, with `powershell.exe` as the fallback. `MANSCRIPT_SHELL` is an explicit diagnostic and test override. Each value selects one executable path; it is not parsed into a command and arguments.

The child receives project tool paths and adapter-specific variables. On Unix, ManScript also supplies a sanitized project prompt. The environment changes exist only in that child process. Exiting it returns to the original terminal environment.

## Files and downloads

Project environments are written below `.manscript/environment`. Runtime providers may download tools below `MANSCRIPT_HOME`, or `~/.manscript` when that variable is unset. ManScript does not use `sudo`; users should still verify repository and package sources before installing dependencies.
