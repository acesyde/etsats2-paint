## Why

TruckPaint has no documentation for players. The README talks to developers (build, mise, CI), and everything a player needs (installing vehicles, building a fleet, drawing, exporting a mod, connecting an AI assistant) has to be found by trial and error. The app speaks English, French, Spanish and German, so its documentation should too, and it should show the app rather than describe it: screenshots and short animations of each feature, in the reader's language.

The app already has the pieces to make this cheap to keep current: a headless harness (`egui_kittest` + wgpu) that drives the real UI, switches its language and renders screens (`tests/screenshots.rs`), and a single source of truth for its colors and fonts (`tp-ui`). The documentation reuses both instead of hand-made captures and a hand-copied look.

## What Changes

- **A player documentation site**, built with Astro Starlight in a new `website/` folder and deployed to GitHub Pages (`https://acesyde.github.io/etsats2-paint/`). It is separate from the developer documents in `docs/`.
- **Four languages**, the app's own: English (default), French, Spanish and German, with a language picker. A page not yet translated falls back to English with a notice. Interface labels quoted in the pages match the app's wording in each language.
- **First version content**, in seven sections:
  - Getting started: installation, first launch, first project;
  - Vehicles: the Vehicle Library, official vehicles and their template pack, custom vehicles;
  - Project and fleet: trucks, variants, trailers;
  - Workshop: vector tools, text, images, gradients, layers;
  - Brand: palette, symbols, styles;
  - Export: textures (PNG, DDS) and the mod (`.scs`);
  - AI assistant (MCP): setting up a client.
- **Generated screenshots and animations.** A local command (`mise run doc:media`) drives the app headlessly, as the screenshot tests do, and renders every documentation image in each of the four languages: screenshots as WebP, and short animations of gestures (drawing a shape, editing a text, exporting…) with a drawn pointer, as MP4/WebM videos. The media are committed; the site build never runs the app.
- **The app's look, in dark and light.** The site uses the app's fonts (Geist for text, JetBrains Mono for values) and its color tokens. The dark theme is the app's palette as is. The light theme, which the app doesn't have, is derived from it and keeps the meaning of each accent (primary action, signal, brand link). The theme follows the reader's system and can be switched. The site's colors are generated from `tp-ui` tokens by the same local command, so they never drift by hand.
- **Deployment and CI.** A workflow builds the site (Node only, no Rust) and publishes it to GitHub Pages on pushes to `main` that touch `website/`. Changes under `website/` count as documentation for the existing CI gating, so they don't run the Rust checks.

Non-goals:
- generating the media in CI (no GPU there; it stays a local step);
- developer documentation (README and `docs/` stay as they are);
- versioned documentation per release, search beyond Starlight's built-in one, comments or analytics;
- a light theme in the app itself;
- Git LFS.

## Capabilities

### New Capabilities
- `user-documentation`: the player documentation site:
  - its languages, the fallback to English, and the match between quoted labels and the app's wording;
  - its content sections;
  - the generation of screenshots and animations per language, their formats, and the check that every referenced medium exists in every language;
  - its dark and light themes, derived from the app's tokens and fonts;
  - its build and deployment to GitHub Pages.

### Modified Capabilities
- `continuous-integration`: files under `website/` are documentation for the Rust checks gating, and a separate job builds and deploys the documentation site.

## Impact

- **New folder `website/`**: Astro Starlight project (pnpm, Node from `mise.toml`), its content in four languages, its committed media and generated `tokens.css`, and the bundled Geist and JetBrains Mono fonts with their licenses.
- **`tp-app` tests**: a new ignored test target for documentation media (scenarios per section, per language), sharing helpers with `tests/screenshots.rs` and `tests/common`; it writes frames and screenshots that a script encodes.
- **`tp-ui`**: the color tokens become readable by the generator (exported to CSS), with no change to the app's look.
- **Tooling**: new mise tasks (`doc:media`, `doc:dev`, `doc:build`); `ffmpeg` becomes a local prerequisite for `doc:media` only.
- **CI**: `scripts/ci-changes.sh` and its self-test treat `website/` as documentation; a new GitHub Pages workflow; the repository's Pages setting must be set to "GitHub Actions".
- **Repository size**: about 100 committed media files (around 15 screenshots and 8 animations per language), kept small by WebP, short videos and a single scale.
- **README** links to the documentation site.
