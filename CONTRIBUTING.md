# Contributing to ManScript

Thank you for considering a contribution.

## Development

```bash
cargo build
cargo test
cargo run -- --help
```

The binary name is `manscript`. When using `cargo run`, the `--` separates Cargo flags from ManScript flags.

## Architecture

- Core project types and the registry live in `src/core` and stay language-agnostic.
- Language and framework behavior belongs in `src/adapters`.
- Runtime detection and downloads belong in `src/runtime` behind `RuntimeProvider`.
- Do not teach core about pip, uv, gem, Bundler, mise, or a particular framework.

To add a language:

1. Implement `LanguageAdapter`.
2. Add a `RuntimeProvider` if needed.
3. Register both in `default_registry()`.
4. Add framework adapters as needed.

## Tests

- Unit tests sit beside their modules and integration tests live in `tests/`.
- CLI tests use `assert_cmd` and must not touch a contributor's real projects or shell startup files.
- Tests that need a network connection or full language installation should be ignored or skip cleanly when tools are unavailable.

## Documentation checklist

For every user-visible change:

- [ ] Update `README.md` when installation, onboarding, support, or the top-level workflow changes.
- [ ] Update the matching page under `docs/` and keep all header/footer navigation links consistent.
- [ ] Update `docs/commands.html` for command names, arguments, prerequisites, mutation behavior, or output changes.
- [ ] Update `docs/config.html` for any `manscript.toml` field, default, validation, or command-execution change.
- [ ] Update `docs/troubleshooting.html` and `docs/shell.html` when setup, detection, PATH, or shell behavior changes.
- [ ] Add an entry under `CHANGELOG.md` → `Unreleased`.
- [ ] Check that every local HTML `href` points to an existing file or page fragment.
- [ ] Preview the static site locally at narrow and wide widths, in light and dark mode, with keyboard-only navigation.
- [ ] Keep pages usable without JavaScript; JavaScript may enhance copy controls but must not hide content.
- [ ] Include explicit dimensions and useful alternative text for new images.

See `docs/README.md` for local preview instructions.

## Pull requests

- Keep the 0.1 scope: no SaaS, AI, Docker orchestration, or extra languages unless agreed.
- Prefer actionable errors over exit codes alone.
- Keep static documentation dependency-free and deployable without a build step.
- Do not commit generated project environments, runtime caches, or credentials.

Contributions are dual-licensed under MIT or Apache-2.0 unless you state otherwise. Follow the [Code of Conduct](CODE_OF_CONDUCT.md).
