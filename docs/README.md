# ManScript documentation site

This directory is a dependency-free static site. There is no build command.

## Local preview

From the repository root:

```bash
python3 -m http.server 8000 --directory docs
```

Open <http://localhost:8000/>. A local server is recommended because Clipboard API behavior can differ when HTML files are opened directly from disk.

You can also use any static-file server. Do not add generated assets, package managers, or a build pipeline for documentation-only changes.

## Navigation

Every HTML page must include the same header and footer destinations, in this order:

1. Home
2. Install
3. Create
4. Commands
5. Config
6. Troubleshooting
7. Shell
8. Uninstall
9. GitHub

Mark the current local page with `aria-current="page"`. Each page also needs a skip link, `<main id="main">`, a unique title and meta description, semantic headings, scoped table headers, and explicit image dimensions.

Before submitting changes, follow every relative HTML `href` and test keyboard focus, clickable code-copy blocks, narrow layouts, dark mode, and reduced motion.

When language support changes, keep the Home, Create, Commands, Config, Install, Troubleshooting, Shell, and Uninstall pages synchronized. Document runtime fallback, dependency commands, project-local cache variables, and write boundaries as behavior rather than presenting language-only adapters as frameworks.

## Deployment

In Vercel, import this repository and set **Root Directory** to `docs`. Do not use the repository root, where Vercel may detect `Cargo.toml` and attempt a Rust build.

The wordmark is `static/manscript-logo.jpg`, the icon is `static/favicon.png`, and the self-hosted Jaro font is `fonts/Jaro.ttf` under the SIL Open Font License (`fonts/OFL.txt`).
