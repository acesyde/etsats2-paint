# TruckPaint

A desktop livery editor for Euro Truck Simulator 2 and American Truck Simulation, written in Rust (egui/wgpu). It is not a general vector editor. `CONTRIBUTING.md` is the full contributor guide; this file is the short version for coding agents.

## Product rules

Every change must respect these (they are also in `openspec/config.yaml`):

- Every project has a vehicle. There is no blank-texture project; a vehicle without a package goes through Custom Vehicle (templates packed locally).
- A project is a fleet of one game (ETS2 or ATS): several trucks with chosen variants and several trailers, exported as one mod.
- Texture layouts (UVs) differ per vehicle. Share across vehicles by elements (palette, symbols with per-texture instances, styles), never by copying a placement. Only variants of the same truck may copy positions.
- A paint job follows the game: a truck has cabins (a main texture each, or one shared main texture), a trailer has one main texture; both have accessory textures shared by the whole vehicle. Trailers have no variants.

## Commands

Everything goes through [mise](https://mise.jdx.dev) (`mise.toml` pins the tools). Run `mise tasks` for the full list.

| Command | What it does |
|---|---|
| `mise run run` | Run the app (debug build) |
| `mise run fmt` / `fmt:check` | Format / check formatting |
| `mise run lint` | Clippy, warnings denied |
| `mise run test` | Unit tests and headless UI tests (`egui_kittest`) |
| `mise run ci` | Everything CI runs, in order. Must pass before a PR |
| `mise run screenshots` | Render the main screens to `target/screenshots/` (needs a GPU) |
| `mise run doc:dev` / `doc:build` | Preview / build the documentation site |
| `mise run doc:media [section]` | Render the doc's screenshots and animations (needs a GPU) |
| `mise run doc:tokens` | Update the site's colors from the app |

## Code layout

Cargo workspace in `crates/`: `tp-core` (domain types, no UI/files/threads), `tp-text`, `tp-render`, `tp-file` (`.truckpaint`), `tp-i18n`, `tp-ui` (design system), `tp-vehicles` (`.tpv` packages), `tp-pack` (`tpv` command), `tp-mcp` (local MCP server), `tp-app` (the desktop app). Official vehicles are described in `catalog/`, sample packages in `examples/vehicles/`.

## How a change is made

- Features and larger changes are planned with OpenSpec in `openspec/` (`specs/` = what is shipped, `changes/` = in progress, `changes/archive/` = done). Use the `/opsx:*` commands. Small fixes need no change.
- Commits follow Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`), with the OpenSpec change name in parentheses at the end when there is one.
- Pull requests follow `.github/pull_request_template.md`: written for players, in basic English, not about files or crates.
- A change that only touches documentation (`*.md`, `openspec/`, `.claude/`, `website/`) skips the Rust CI jobs.

## Translations

The app is in English, French, Spanish and German. Every string lives in `crates/tp-i18n/locales/<lang>/*.ftl` (Fluent); a new string goes into all four languages with the same message id.

- English: Title Case for commands, menu items, buttons and dialog titles (small words lowercase unless first); sentence case for labels, hints, messages and tooltips. American spelling.
- Other languages follow their own usage (sentence case in French, Spanish and German, except German nouns) and use the game's own terms when it has them.
- German is the longest: check that screens still fit.

## Documentation site

Player documentation is an Astro Starlight site in `website/`, in the same four languages, published at <https://truckpaint.ace-dev.me/>. Pages are `website/src/content/docs/<lang>/<section>/<page>.mdx`; sections are listed in `website/astro.config.mjs`.

- Write the English page first, then the three translations. The files and `sidebar.order` match across languages.
- Quote an interface label with `<Ui id="message-id" />` (never type it by hand): it shows the app's own text in the page's language, so the id must exist in all four locales. Use `<Shot name="…" alt="…" />` for a screenshot and `<Clip name="…" label="…" />` for an animation. The build fails on an unknown id or a missing medium.
- Screenshots and animations are rendered from the app by `crates/tp-app/tests/doc_media.rs` and committed under `website/public/media/<lang>/`. After changing a documented screen, run `mise run doc:media <section>`.
- Site colors come from the app's design tokens; a test fails when they are stale (`mise run doc:tokens`).
- `vehicles/package-format.mdx` has no fr/es/de translation yet; Starlight falls back to the English page with a "not translated" notice. Don't treat it as an error, but ask before translating it.

Specialized agents live in `.claude/agents/`: `docs-writer`, `docs-translator` and `docs-drift-checker` for the documentation, and `rust-ui-tester` to run the format, lint and test checks and report only the failures.
