# Later (not Django)

Django in-project create now wires **apps** (INSTALLED_APPS, urls, hello view) and **models** (admin register, `makemigrations` + `migrate` via the project env). Everything below is parked so we can do it with the same security rules: argv only, no shell, no sudo, writes only under the project root, isolated env binaries.

## Flask

- Templates (and a templates folder convention)
- Extensions (only via `manscript install` / pinned requirements — never a random pip from a prompt)
- App factory layout

## FastAPI

- Auth (do not generate JWT/secret defaults)
- Pydantic schemas as a first-class generator
- Extra modules beyond a router

## Rails

- Other `rails generate` kinds beyond scaffold / resource / controller / model
- Still only through project `bin/rails` / env gems

## Sinatra

- Larger layouts (modular `Sinatra::Base`, extra folders)

## Python / Ruby only

- Optional extra file/module generator (there is no framework “app”)

## PHP, Go, Rust, C# (language-only)

C, C++, and Java now have thin language-only adapters (system toolchain shims, hello file, `manscript build` / `run`, no package managers). Still parked:

- **PHP** — interpreter + isolated env story (no Laravel in this slice)
- **Go** — `go` toolchain / modules
- **Rust** — `rustc` / Cargo
- **C#** — `dotnet` SDK

Same argv/env rules when those land: no shell, no sudo, writes only under the project root.

When any of these land: same process security as `manage.py` / `bin/rails` today.
