## Context

`tp-pack`'s extractor (`crates/tp-pack/src/extract/`) reads the game through ConverterPIX, one process per call (`Sources::command`). Measured on the local ETS2 1.61.1.9 install (115 archives):

- `Sources::locate` handles a path missing from its file map by listing the parent folder in **every** archive (`--list-directory`, one call each). Nothing remembers which folders were already listed, so each new miss in an already listed folder (an absent optional file, a sibling not yet seen) lists the same folder in the 115 archives again. One truck (`scania.r_2016`) makes about 11,800 converter calls.
- A listing costs 3 to 30 ms depending on the archive. A model conversion (`-m`, all 115 archives mounted) costs about 0.25 s. A full batch makes about 140,000 calls, 9,955 of them model conversions, in about 70 minutes.
- Each batch reserves `target/game-extraction/ets2-<stamp>-<pid>-<n>/`, which holds the extracted definitions per archive (`archive-<i>/`), converted models (`models/`) and one `converter-<n>.log` per call. That is about 1 GB for a one-vehicle batch and about 12 GB for a full batch (11 GB of converted `.pim`/`.pit` text), and it is never removed.
- `--list-directory` prints `[D] <path>` and `[F] <path>` entries. An absent folder gives `readDir returned null`, which `command` already accepts for listings.

## Goals / Non-Goals

**Goals:**
- Far fewer listing calls, with exactly the same file map: same files, same archives per file, same order. Hence the same provenance, precedence, packages and report evidence.
- No conversion of a model already converted from the same game files by the same converter.
- No work folder left behind by a successful batch.

**Non-Goals:**
- Parallel model conversion: the first batch after a game update still spends about 40 minutes converting.
- Caching definitions or listings between batches: they are cheap once listings are memoized.
- Cleaning work folders from earlier batches, or a flag to keep the work folder of a successful batch.

## Decisions

1. **Memo of listed folders per archive.** `Sources` keeps a `listed: BTreeMap<(usize, String), Option<Vec<Entry>>>` (archive index, folder). `None` means the folder is absent from that archive. A folder is listed at most once per archive per batch. `inventory_folder` uses the same memo, so `/def/vehicle/truck` listed by the inventory is never listed again. Alternative rejected: remembering only "folder already scanned in all archives". That avoids repeats but still pays 115 listings for each new folder.

2. **Top-down pruning.** To list folder `F` in archive `i`, first get the listing of `parent(F)` in `i` from the memo, recursively up to `/`. If `parent(F)` is absent from `i`, or it does not list `F` as a `[D]` entry, then `F` is absent from `i` with no call. Most folders are then listed only in the few archives that contain their whole path. The root `/` is listed once per archive. Risk: an archive whose listings omit intermediate `[D]` entries would hide files that a direct listing would find. Mitigation: a full-batch comparison of the file map (decision 4) before relying on it. If a real archive needs it, fall back to a direct listing for that archive (kept as a one-line rule, not a mode).

3. **File map built from memoized listings, unchanged semantics.** `[F]` entries of a listing add `(path → archive index)` exactly as today, kept sorted by archive index, so "last mounted wins" and `mounted_sources` are unchanged. Extraction (`--extract-file`, `--extract-directory`) and model conversion calls are unchanged.

4. **Proof of equivalence.**
   - **Fake converter:** it records every call to a file. Tests assert that a folder is listed at most once per archive, that no listing goes to an archive lacking the parent folder, and that the packages and report evidence are unchanged.
   - **Local game:** one full batch with `--previous` on the last batch. All 69 packages must be byte-identical, and the `sources` and `name_sources` evidence equal. Call count and duration are recorded in `docs/game-extraction-plan.md`.

5. **Model cache shared by batches.**
   - **Location:** `target/game-extraction/cache/` holds `key.json` and `models/`, with the same tree as the work folder's `models/` today (`<model path>.pim` and `.pit`).
   - **Key:** a JSON list of `[archive file name, size, modification time in ns]` for every mounted archive, in mount order, plus the converter's canonical path, size and modification time. No hash, so no new dependency.
   - **Batch start:** read `key.json`. If it is missing or differs, remove `models/` and write the new key. One cache only: a game update replaces it, so old caches do not pile up.
   - **Conversion:** `unit_geometry` looks in the cache first. On a miss, the converter writes into the batch work folder as today. Once `command` succeeds, the `.pit`, then the `.pim`, are moved into the cache with `rename` (same filesystem under `target/`). A model counts as cached only when both files are present, and the `.pim` arrives last, so an interruption never leaves a half-cached model.
   - **Report:** `model_cache` holds `directory`, `converted` and `reused`.
   - Alternatives rejected:
     - Content hashes of the archives (several GB to read at every start, plus a new dependency).
     - One cache folder per key (old caches pile up after every game update).
     - Keying on the game version only (a DLC update or a replaced converter would reuse stale models).

6. **Work folder cleanup.** After `report.json` is published, `run` removes the batch work folder with `remove_dir_all` when `failed == 0`. The report states `work_directory` and `work_directory_kept` (written before removal, from the same `failed == 0` test). A removal error is reported on stderr but does not fail a batch whose packages and report are published. The batch is complete; only disk space is lost. When any vehicle fails, nothing is removed, so the `see <log>` paths in failure messages stay valid. Alternative rejected: deleting only converter logs and keeping definitions. That still leaves most of the 1 GB, and nothing reuses it across batches.

## Risks / Trade-offs

- [An archive's listing lacks intermediate folders, so pruning hides files] → The full-batch byte and evidence comparison catches it. The fake converter cannot prove it for real archives.
- [Removing the work folder deletes intermediates someone wanted to inspect] → Only on success, where nothing is unexplained. Failures keep everything. Running a single vehicle that fails keeps its folder.
- [Archive replaced with the same size and modification time] → Steam and the game's updates change both. A forced rebuild means deleting `target/game-extraction/cache/` by hand, which is documented in the extraction notes.
- [Two batches with different keys at the same time clear each other's cache] → Not a supported use. The batch that loses a model mid-run fails that vehicle with a converter or missing-file error, and no package is published from a half state.
- [Cache size] → A full batch converts 9,955 models whose `.pim`/`.pit` text weighs about 11 GB. The cache holds that once per game version and is cleared on the next key change; today every full batch leaves its own 11 GB in its work folder.
- [Memory: the memo holds every listing of a batch] → A few tens of thousands of short paths, at most a few MB.
- [First batch after a game update stays long] → Measured: 55 minutes instead of 65, since conversions (9,978 at about 0.25 s) dominate once listings are memoized. A batch on the same game then takes 4 minutes. Parallel conversion is the next lever if the first batch must be shorter.

## Migration Plan

No migration. Existing work folders under `target/game-extraction/` are left as is; players and maintainers can delete that folder by hand.

## Open Questions

None.
