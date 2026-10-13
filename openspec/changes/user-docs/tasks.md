## 1. Site skeleton (website/)

- [x] 1.1 Create the Astro Starlight project in `website/` with pnpm (lockfile committed), locales en (default), fr, es, de, `site`/`base` for `https://acesyde.github.io/etsats2-paint/`, and a root redirect to `/en/` (design Decision 1); verify `pnpm build` succeeds and `dist/` has `en/`, `fr/`, `es/`, `de/` and a root page redirecting to `/en/`.
- [x] 1.2 Add the sidebar with the seven sections in order and their labels in the four languages, with one placeholder page per section in English only; verify user-documentation "Sidebar" in each locale and "Missing translation" (the `/es/` URL of a placeholder shows English with Starlight's notice).
- [x] 1.3 Add the mise tasks `doc:dev` and `doc:build` (design Decision 9); verify user-documentation "Local preview" (`mise run doc:dev` serves all four locales with live reload) and that `mise run doc:build` gives the same `dist/` as `pnpm build`.

## 2. Theme, tokens and fonts

- [x] 2.1 Add the `tp-ui` example `tokens_css` that writes `website/src/styles/tokens.css` with the dark tokens of `tokens.rs` as `--tp-*` properties, and the derived light palette following design Decision 6, failing when a text or accent pair is below WCAG AA (checked with `contrast.rs`); add `mise run doc:tokens`; verify the dark values equal `tokens.rs` and the example fails when a light text color is made too pale.
- [x] 2.2 Add the non-ignored `tp-ui` test that compares the committed `tokens.css` with the example's output and tells to run `mise run doc:tokens`; verify user-documentation "Tokens follow the app" by changing `SURFACE_1`, seeing the test fail, regenerating and seeing it pass (then revert).
- [x] 2.3 Copy Geist (Regular, Medium, SemiBold) and JetBrains Mono (Regular, Medium) with their licenses to `website/src/fonts/`, declare them with `@font-face`, and write `theme.css` mapping `--tp-*` to Starlight's `--sl-color-*` and font variables, with the border and shadow for media in the light theme; verify user-documentation "Dark system" and "Switching theme" in a browser, that text uses Geist and code uses JetBrains Mono (devtools, computed font), and that no font is loaded from a third-party host.

## 3. Shared test helpers and media generator

- [x] 3.1 Move the scene and harness helpers of `tests/screenshots.rs` used by both targets (`demo_scene`, `demo_brand`, `add_text`, `wait_for_thumbnails`, `wait_for_images`, `open_menu`, `settle_dialog`) to `tests/common/scenes.rs` (the canvas tests keep their drag helpers: the animation driver of 3.3 has its own), with no change of behavior; verify `mise run test` passes and `mise run screenshots` still writes the same set of files.
- [x] 3.2 Create the ignored test target `tests/doc_media.rs`: one function per scenario, rendering in `Language::ALL` at 1280 × 800 points and 1.5 pixels per point, with the drawn Windows/Linux title bar, to `target/doc-media/<lang>/<name>.png`, one `section_<name>` test per section so a section can be run alone (design Decision 2); verify one screenshot scenario renders four PNGs whose interface is in the four languages.
- [x] 3.3 Add the animation driver: timed actions (move to a point or a widget by translated label, press, drag, release, type, wait), eased pointer interpolation at 30 fps, one harness step and render per frame, the pointer sprite and press ring composited on each frame, an optional crop region, and a 1 s hold at the end (design Decision 3); verify a "draw a rectangle" scenario writes numbered frames where the pointer moves, the rectangle grows during the drag, and two runs give identical frames.
- [x] 3.4 Write `scripts/doc-media.sh` and `mise run doc:media [section]`: check that ffmpeg is installed (clear message otherwise), run `doc:tokens`, copy the fonts, run the test target, encode screenshots to WebP (q85) and frames to H.264 MP4 (30 fps, yuv420p, faststart, no audio) with a WebP poster into `website/public/media/<lang>/`, and write `manifest.json` (version, commit, dirty flag, date, files per language) (design Decisions 2 and 4); verify on one section that the files appear for the four languages and that an unrelated section's files are left untouched.

## 4. Documentation components

- [x] 4.1 Add `<Shot>` and `<Clip>`: resolve the medium for the page's language under the base path, throw at build time naming the page and file when it is missing; `<Clip>` plays muted, looped and without controls, and without autoplay but with controls and the poster under reduced motion (design Decision 4); verify user-documentation "Missing medium" (build fails with the page and file named), "Reduced motion" (emulated in devtools), and "Page shows its language's screenshot".
- [x] 4.2 Add `<Ui id="…">` reading `crates/tp-i18n/locales/<lang>/*.ftl` with `@fluent/bundle` (isolation marks off), with optional arguments, failing the build on an unknown id or a missing argument (design Decision 5); verify user-documentation "French label" (the French page shows the same text as the French Export menu item) and "Unknown label" (build fails naming the page and id).

## 5. Media scenarios and English content

Each task writes the section's scenarios in `doc_media.rs`, runs `mise run doc:media <section>`, and writes the English pages using `<Shot>`, `<Clip>` and `<Ui>`. Verify each with `mise run doc:build` and by reading the pages in the browser in both themes: the media show the step the text describes, and the wording follows the product rules (user-documentation "Content sections").

- [x] 5.1 Getting started: installation (per platform, graphics troubleshooting from the README), first launch, first project; screenshots of the home screen and New Project, and the "create a new project" animation.
- [x] 5.2 Vehicles: Vehicle Library, official vehicles and the template pack dialog, custom vehicles; three screenshots (no third-party templates shown).
- [x] 5.3 Project and fleet: the Project space, adding trucks with variants and trailers, one game per project; two screenshots.
- [x] 5.4 Workshop: vector tools, text, images, gradients, layers; animations "draw a rectangle" (user-documentation "Drawing a rectangle"), "edit a text", "move and resize" and "draw a path with the Pen", plus screenshots of the panels, layers, texts and images, shapes and paths, and gradients.
- [x] 5.5 Brand: palette, symbols, styles, and why sharing goes by elements and not by placement; the "apply a swatch" and "apply a style" animations, and screenshots of the Resources tab and the Brand space.
- [x] 5.6 Export: textures (PNG, DDS) and the mod (`.scs`, where it is written, enabling it in the game); one screenshot of Export Mod and the "export the mod" animation.
- [x] 5.7 AI assistant (MCP): turning it on in Preferences, adding it to Claude Code, Claude Desktop and other clients (from the README), what it can and can't do; one screenshot of the Preferences section.

## 6. Translations

- [x] 6.1 Translate every page to French, following the Wording rules of `docs/roadmap.md`; verify `mise run doc:build` passes and no French page shows the "not translated" notice.
- [x] 6.2 Translate every page to Spanish; same check for `/es/`.
- [x] 6.3 Translate every page to German; same check for `/de/`, and that the German screenshots show no clipped labels.

## 7. CI and deployment

- [x] 7.1 Add `website/*` to the documentation patterns of `scripts/ci-changes.sh` (and its header comment), with a case in `scripts/ci-changes-test.sh`; verify `mise run ci:changes:test` passes and continuous-integration "Documentation site change" and "Unknown file kind" hold.
- [ ] 7.2 Add `.github/workflows/docs.yml` (paths `website/**`, `crates/tp-i18n/locales/**`, the workflow itself; mise-action for Node and pnpm; build on pull requests; build and deploy to Pages on `main` with `upload-pages-artifact` and `deploy-pages`; actions pinned by SHA; concurrency group `pages`) (design Decision 8); verify on a pull request that the build job runs and fails when a medium is removed (continuous-integration "Broken page in a pull request"), and that a Rust-only change doesn't trigger it ("No site change").
- [ ] 7.3 Set the repository's Pages source to "GitHub Actions", merge, and check continuous-integration "Deploy on merge" and user-documentation "Publishing a page change": the site is online at `https://acesyde.github.io/etsats2-paint/` in four languages, with images, videos, fonts and styles loading under the base path.

## 8. Wrap-up

- [x] 8.1 Link the documentation site from the README (top and MCP section), and add a Documentation paragraph to its Development section: `doc:dev`, `doc:build`, `doc:media` (GPU and ffmpeg), and the rule to regenerate the sections whose UI a change touches; verify the links resolve.
- [x] 8.2 Measure the full media set (count and size per language) and record it in design.md under Repository growth; verify it is under the 15 MB estimate or note what was done about it.
- [ ] 8.3 Update `docs/roadmap.md`: move `user-docs` from Next changes to Shipped with a short summary.
