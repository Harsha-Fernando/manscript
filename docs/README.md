# Docs site (Vercel)

Static HTML. No GitHub Pages.

In the Vercel dashboard, import this GitHub repo and set **Root Directory** to `docs`. Do not use the repository root: Vercel would see `Cargo.toml` and try to build Rust.

After that, deploy. There is no build command.

The header wordmark is `static/manscript-logo.jpg`. The tab icon is `static/favicon.png`. Headings use self-hosted **Jaro** (`fonts/Jaro.ttf`, SIL OFL — see `fonts/OFL.txt`).
