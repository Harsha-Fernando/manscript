# Contributing to ManScript

Thank you for considering a contribution.

## Development

```bash
cargo build
cargo test
cargo run -- --help
```

The binary name is `manscript`.

## Architecture

- Core types and registry live in `src/core`. They must stay language-agnostic.
- Language and framework behavior belongs in `src/adapters`.
- Runtime download/detect belongs in `src/runtime` behind `RuntimeProvider`.
- Do not add `PythonManager` / `RubyManager` to core.
- Do not teach core about pip, uv, gem, Bundler, or mise.

To add a language later:

1. Implement `LanguageAdapter`.
2. Optionally add a `RuntimeProvider`.
3. Register both in `default_registry()`.
4. Add framework adapters as needed.

## Tests

- Unit tests sit next to modules and in `tests/`.
- CLI tests use `assert_cmd` and must not touch the contributor's real projects.
- Tests that need network or a full language install should be `#[ignore]` or skip when tools are missing.

## Pull requests

- Keep the 0.1 scope: no SaaS, AI, Docker orchestration, or extra languages unless agreed.
- Prefer clear errors over exit codes alone.
- Update `CHANGELOG.md` for user-visible changes.

Contributions are dual-licensed under MIT or Apache-2.0, the same as the rest of this repository, unless you state otherwise.

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
