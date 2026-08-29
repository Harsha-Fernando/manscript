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

- Run commands through a shell
- Invoke `sudo` or escalate privileges
- Modify system files silently
- Overwrite an existing project without confirmation
- Send project contents to a remote service

Commands in `manscript.toml` are treated as argv and validated. Treat that file as trusted configuration for the local project, similar to a Makefile.
