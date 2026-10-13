## Context

See proposal.md for the why. The facts in the repository that shape the approach:

- **The app can already be driven and rendered headlessly.** `crates/tp-app/tests/screenshots.rs` builds an `egui_kittest` harness with the wgpu renderer (`common::wgpu_harness_with(prefs, size)`), sets the language through `Prefs::set_language`, clicks widgets by their translated label (`tp_i18n::tr(id)`), and saves `harness.render()` as PNG. It is `#[ignore]` because it needs a GPU. `tests/canvas.rs` has helpers that inject pointer drags.
- **Four languages**, as Fluent files in `crates/tp-i18n/locales/<lang>/*.ftl` (en, fr, es, de), read with `fluent-bundle`.
- **The app has one theme, dark.** `tp-ui/src/theme.rs` forces `ThemePreference::Dark`, and `tp-ui/src/tokens.rs` holds the palette by meaning: surfaces `SURFACE_0..4` (`#1C1C1F` to `#44444A`), text levels, and three accents with one meaning each (`ACCENT_PRIMARY` white, `SIGNAL` red, `LINK` blue). `tp-ui/src/contrast.rs` computes WCAG luminance and ratios.
- **Fonts are bundled** in `assets/fonts/`: Geist (interface) and JetBrains Mono (values), both under the SIL OFL, which allows redistribution with the license.
- **mise already pins Node 24 and pnpm**, and CI installs tools with `jdx/mise-action`. CI runs on `ubuntu-latest` without a GPU, and `scripts/ci-changes.sh` skips the Rust jobs when only documentation changed (`*.md`, `openspec/`, `.claude/`).
- **The repository is `acesyde/etsats2-paint`**, so its Pages URL is `https://acesyde.github.io/etsats2-paint/`.

## Goals / Non-Goals

**Goals:**
- The documentation shows the real app, in the reader's language, and costs one command to bring up to date after a UI change.
- The site never shows an interface label, screenshot or color that disagrees with the app, and fails its build rather than doing so silently.
- Building and deploying the site needs neither Rust nor a GPU.
- The repository stays reasonably small despite committed media.

**Non-Goals:**
- Generating media in CI (software Vulkan through lavapipe could work, but it is not needed for a local workflow).
- A light theme in the app, or any change to the app's look.
- Versioned docs, translations of the developer documents.

## Decisions

### 1. Astro Starlight in `website/`

Starlight gives what the site needs out of the box: locales under `/en/`, `/fr/`, `/es/`, `/de/`, a language picker, fallback to the default locale with a "not translated yet" notice, a dark/light/auto theme switch remembered per reader, built-in search (Pagefind), and MDX for custom components. Node and pnpm are already pinned in `mise.toml`.

- Content: `website/src/content/docs/<lang>/<section>/<page>.mdx`. English is the default locale; the root URL redirects to `/en/`.
- Config: `site: "https://acesyde.github.io"`, `base: "/etsats2-paint"`. Every link and asset goes through Starlight or `import.meta.env.BASE_URL` so the base path holds.
- *Alternative: mdBook.* It matches the Rust stack, but multilingual books need gettext tooling and one book per language, with no shared picker or fallback.
- *Alternative: Zola.* It has native multilingual support, but no documentation theme with search, sidebar and theme switch as complete as Starlight's, so more would be built by hand.

`website/` is separate from `docs/`, which stays for developer documents (roadmap, package format).

### 2. Media are generated locally by a new ignored test target, then committed

A new test target `crates/tp-app/tests/doc_media.rs`, `#[ignore]` like the screenshots, holds one function per scenario. Each scenario builds a fixed scene and renders it once per language in `Language::ALL`.

- **Shared helpers.** The scene builders and waits that `screenshots.rs` already has (`demo_scene`, `demo_brand`, `add_text`, `wait_for_thumbnails`…) move to `tests/common/scenes.rs` so both targets use them. `screenshots.rs` stays the QA tool (two languages, two scales, long-text mode). `doc_media.rs` is the documentation tool (four languages, one scale, framed for reading).
- **Platform look.** Documentation media show the drawn title bar of Windows and Linux, not the macOS native menu bar, since most ETS2/ATS players are on Windows.
- **Content.** Scenes use the bundled sample truck (and a trailer), never an official vehicle built from the third-party template pack, which is not ours to redistribute. The official vehicles page shows the dialogs, not the pack's templates.
- **Size and scale.** Window 1280 × 800 points rendered at 1.5 pixels per point (1920 × 1200 px). One scale only, to keep the media count down.
- **Output.** Raw PNG frames go to `target/doc-media/` (not committed). A script (`scripts/doc-media.sh`, run by `mise run doc:media`) runs the test target, then encodes into `website/public/media/<lang>/`:
  - screenshots: WebP, lossy quality 85 (around 100–200 KB for a full window);
  - animations: H.264 MP4, 30 fps, `yuv420p`, `+faststart`, no audio, cropped to the region of interest when the scenario declares one, plus a WebP poster from the first frame.
- **Partial runs.** `mise run doc:media -- <section>` regenerates one section only, so a UI change in the Workshop doesn't rewrite every file in the repository.
- `ffmpeg` encodes both formats. It is pinned as a tool of the `doc:media` mise task only (`tools = { ffmpeg = … }`), so mise installs it when the task runs and CI, which only installs the top-level tools, never does.

*Alternative: GIF animations.* They are larger and limited to 256 colors, which bands the app's gradients and the livery colors. Rejected.
*Alternative: MP4 and WebM.* Every current browser plays H.264 MP4; a second format doubles the files for nothing. MP4 only.

### 3. Animations: scripted gestures with a drawn pointer

`egui_kittest` renders one image per step and has no OS cursor. An animation scenario is a list of timed actions: move the pointer to a point or a widget (found by its translated label), press, drag, release, type text, wait. A small driver interpolates the pointer between points (eased, at 30 fps), feeds the events to the harness, runs one step per frame, renders, and composites a pointer sprite (an arrow, with a ring briefly shown on press) at the pointer position onto the frame before saving it.

- Time is driven by the harness's fixed step, so the same scenario gives the same frames from one run to the next on the same machine.
- Each clip is short (3 to 8 s) and ends with a 1 s hold on the result before looping.
- Background work that repaints (template rendering, thumbnails) is awaited before recording starts, as `wait_for_thumbnails` does for screenshots.

### 4. Manifest and completeness check

`doc:media` writes `website/public/media/manifest.json`: the app version, the commit (with a `dirty` flag when the working tree had changes), the generation date, and the list of files per language.

Pages use two MDX components, `<Shot name="workshop/text" alt="…" />` and `<Clip name="workshop/draw-rectangle" />`. They resolve the file for the page's language and **throw at build time** when it is missing, naming the page and the file. So an untranslated or forgotten medium fails the site build (and the pull request check) instead of showing a broken image.

`<Clip>` renders `<video autoplay loop muted playsinline poster>` with no controls. Under `prefers-reduced-motion: reduce` it renders without autoplay and with controls, showing the poster.

### 5. Interface labels read from the app's Fluent files

Pages quote labels with `<Ui id="menu-export-mod" />`. The component reads `crates/tp-i18n/locales/<lang>/*.ftl` at build time with `@fluent/bundle` (the JavaScript implementation of the same Fluent format, with isolation marks off) and renders the message for the page's language, styled as an interface label. An unknown id fails the build, naming the page and the id. Trailing "…" are kept, as in the menus.

The labels follow the `.ftl` files without regeneration. Because of that, a change to `crates/tp-i18n/locales/` also triggers the documentation build check (Decision 8).

*Alternative: export the labels to JSON in `doc:media`.* That adds a regeneration step for every wording change and can drift. Reading the source is simpler.

### 6. Theme: tokens exported from `tp-ui`, light palette derived next to them

A `tp-ui` example, `cargo run -p tp-ui --example tokens_css` (run by `doc:media`, also runnable alone as `mise run doc:tokens`), writes `website/src/styles/tokens.css`:
- `:root[data-theme="dark"]`: the app's tokens as `--tp-*` custom properties (surfaces, text levels, borders, accents), with the exact values of `tokens.rs`;
- `:root[data-theme="light"]`: a light palette defined in the example, next to the mapping, with these rules: surfaces mirrored to light neutrals with the same low saturation; text levels inverted; `ACCENT_PRIMARY` becomes near-black (the primary action keeps its contrast against its surface); `SIGNAL` and `LINK` keep their hue and get a lower lightness for contrast on light surfaces. The example checks every text and accent pair with `contrast.rs` and fails if one is below AA (4.5:1 for text).

A hand-written `website/src/styles/theme.css` maps the `--tp-*` properties to Starlight's `--sl-color-*` and `--sl-font*` variables. It is the only place where the site's look meets Starlight.

A unit test in `tp-ui` (not ignored, so it runs in the normal CI) compares the committed `tokens.css` with what the example would write, and fails with "run `mise run doc:tokens`" when a token changed without regenerating. That keeps the dark theme equal to the app's without relying on memory.

**Fonts.** Geist (Regular, Medium, SemiBold) and JetBrains Mono (Regular, Medium) are copied from `assets/fonts/` to `website/src/fonts/` with their license files, and declared with `@font-face` (`font-display: swap`) in `theme.css` with relative URLs, so the build fingerprints them and serves them under the base path. They stay TTF: around 0.9 MB in total, loaded on demand, and the same files as the app. Converting them to WOFF2 would save about half of that if it ever matters.

Screenshots and animations stay dark in the light theme, since that is how the app looks. In the light theme they get a 1 px border and a soft shadow so they don't float.

### 7. Content and wording

- Seven sections as listed in the proposal, in the sidebar in that order, with translated labels per locale.
- English is written first. The French, Spanish and German pages are translations of it, reviewed against the app's own wording (Decision 5 handles labels; prose follows the Wording rules of `docs/roadmap.md`, including American English and sentence case for fr/es/de).
- Pages explain product rules where the reader meets them: a project is a fleet of one game; sharing across vehicles goes through the palette, symbols and styles, never by copying a placement; trailers have no variants.

### 8. CI and deployment

- **Gating.** `scripts/ci-changes.sh` adds `website/*` to the documentation patterns, and its self-test (`scripts/ci-changes-test.sh`) gets a case for it. A pull request that only edits pages or media skips the Rust jobs. A change to `crates/tp-i18n/locales/` stays code (it also changes the app).
- **New workflow `.github/workflows/docs.yml`:**
  - triggers on pull requests and pushes to `main` with `paths: [website/**, crates/tp-i18n/locales/**, .github/workflows/docs.yml]`, and on `workflow_dispatch`;
  - installs Node and pnpm with `jdx/mise-action` (same pin as CI, actions pinned by SHA like `ci.yml`), runs `pnpm install --frozen-lockfile` and `pnpm build` in `website/`;
  - on `main` only, uploads `website/dist` with `actions/upload-pages-artifact` and deploys with `actions/deploy-pages` (permissions `pages: write`, `id-token: write`, concurrency group `pages`).
- It is separate from "CI result": a broken docs build doesn't block a code fix, and a docs-only pull request still shows its own docs check.
- The repository's Pages source must be set to "GitHub Actions" once, by hand.

### 9. mise tasks

- `doc:media [section]`: tokens, fonts and media, then the manifest. Needs a GPU and ffmpeg.
- `doc:tokens`: `tokens.css` only (no GPU).
- `doc:dev`: `pnpm dev` in `website/` (local preview with live reload).
- `doc:build`: `pnpm build` in `website/`, the same command as CI.

## Risks / Trade-offs

- **[Media drift from the app]** Media are only as fresh as the last `doc:media` run. → The manifest records the commit they came from. Pull requests that change the UI of a documented screen should regenerate its section; this is noted in the README's Development section. Labels and colors cannot drift (Decisions 5 and 6).
- **[Repository growth]** Each regeneration adds a new version of the changed files to history. → WebP and MP4, one scale, cropped clips, per-section regeneration. Measured on the first full set: 38 files per language (22 screenshots, 8 animations with their posters), about 3.3 MB per language and 13 MB in all, under the 15 MB estimate. If history grows too much, media can move to a separate `gh-pages`-like branch later.
- **[GPU differences]** Two machines can render slightly different pixels (driver rasterization), so a regeneration on another machine rewrites unchanged screens. → Regenerate per section, and only the sections whose UI changed.
- **[Translation effort]** Three languages to keep in step with English. → Starlight's fallback shows English with a notice instead of a missing page; labels and media are translated automatically, only prose needs translating.
- **[Fluent features in labels]** A label that takes arguments (counts, names) can't be quoted as is. → `<Ui>` accepts arguments for the rare cases, and fails the build when a required one is missing.
- **[Pointer realism]** A drawn pointer doesn't show OS-specific cursors (resize, text). → One neutral arrow; the gesture's effect on the canvas carries the meaning.

## Open Questions

- The exact light palette values are set during implementation, within the rules of Decision 6, and reviewed on screenshots of both themes.
