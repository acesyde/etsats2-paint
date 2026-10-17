## 1. Measure today

- [x] 1.1 Make the fake converter record each call (action, path, mounted archive) to a file, and add a test helper that reads it; verify with a test asserting today's behavior: listing a folder for a missing file goes to every archive.

## 2. Fewer listings

- [x] 2.1 Remember each folder listing per archive for the batch, absent folders included, and use it in both the inventory and file lookup; verify with the call log that no folder is listed twice in the same archive, and that all existing extraction tests pass unchanged.
- [x] 2.2 List a folder in an archive only when that archive's listing of the parent folder contains it, walking down from the root; verify with the call log that a fixture archive without `/vehicle` gets no listing below `/vehicle`, that a file present in two archives keeps both sources in order (provenance test), and that all extraction tests pass.

## 3. Model cache

- [x] 3.1 Keep converted models in `target/game-extraction/cache/` with a `key.json` built from the mounted archives and the converter (name or path, size, modification time); clear the cache when the key differs; verify fixtures: a second batch on the same fake archives makes no `-m` call and publishes byte-identical packages, and touching one fake archive clears the cache and converts again.
- [x] 3.2 Convert a cache miss into the batch work folder and move `.pit` then `.pim` into the cache only after a successful conversion; record `model_cache` (`directory`, `converted`, `reused`) in the report; verify a fixture with a failing conversion leaves nothing in the cache and the report counts match the call log.

## 4. Work folder cleanup

- [x] 4.1 After publishing the report, delete the batch work folder when no vehicle failed, and record `work_directory` and `work_directory_kept` in the report; verify fixtures: a successful batch (including a skipped not-paintable trailer) leaves no work folder, and a batch with one failed vehicle keeps it with the converter log named in the failure.
- [x] 4.2 Update the official vehicles documentation page in English, then French, Spanish and German: intermediates and converter logs are kept under `target/` only when a vehicle fails, and converted models are reused from a cache until the game or the converter changes; verify `mise run doc:build` passes.

## 5. Validate on the game

- [x] 5.1 Run a full batch on the local ETS2 installation with `--previous` on the last full batch; verify all packages are byte-identical, the `sources` and `name_sources` evidence are equal, and the work folder is removed. Record the converter call count and duration before and after in `docs/game-extraction-plan.md`.
- [x] 5.2 Run the same full batch a second time; verify it reuses every model (`converted` is 0), its packages are byte-identical to 5.1, and record its duration.
- [x] 5.3 Run `mise run ci` and verify format, lint, tests and build pass; report any GPU or game-only checks that could not run.
