# Security Policy

## Supported versions

The 0.1.x line is the current development release.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Email or otherwise contact the maintainers privately with:

- A description of the issue
- Steps to reproduce
- Impact (for example: unexpected subprocess execution, path traversal, privilege concerns)

## Design notes

ManScript executes language toolchains on behalf of the user.

It will not:

- Run configured project commands through a shell or interpolate them into shell strings
- Invoke `sudo` or escalate privileges
- Modify system files silently
- Modify shell startup files or the parent process environment
- Overwrite an existing project without confirmation
- Send project contents to a remote service

Commands in `manscript.toml` are treated as argv and validated. Treat that file as trusted configuration for the local project, similar to a Makefile.

`manscript shell` is the explicit exception that launches the user's interactive shell. It starts the shell executable directly, with no `-c` command string, and inherits stdin, stdout, and stderr. Project tool paths and adapter-specific variables are applied only to that child process. The project name is sanitized before it is placed in the prompt so prompt expansion cannot execute configuration content.

On Unix, the child executable normally comes from `SHELL`; on Windows it normally comes from `COMSPEC`. `MANSCRIPT_SHELL` is an explicit test/diagnostic override. These values select one executable path only—they are never parsed as command strings or combined with arbitrary arguments.
