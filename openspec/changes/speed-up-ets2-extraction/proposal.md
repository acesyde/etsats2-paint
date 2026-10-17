## Why

A full `tpv extract` batch (26 trucks, 58 trailers) takes about 70 minutes, and each batch leaves its intermediates behind (about 12 GB for a full batch, 11 GB of them converted models) that are never cleaned (55 GB after 16 local runs). Most of the time goes to repeated archive listings: to find a file it does not know yet, the extractor lists the parent folder in all 115 mounted archives, and does it again for every new miss. A single truck makes about 11,800 converter calls. The other half of the time goes to about 10,000 model conversions (0.25 s each), redone identically by every batch on the same game. Publishing a new game version means rerunning batches, so the wait and the disk use are paid every time.

## What Changes

- Folder listings are remembered per archive for the whole batch: a folder is listed at most once in each archive.
- A folder is listed only in the archives that contain its parent folder, walking down from the root, instead of in every mounted archive.
- File provenance, archive precedence, packages and report evidence stay identical. Truck and trailer packages must stay byte-identical to the previous batch.
- Converted models are kept in a cache shared by batches, `target/game-extraction/cache/`. A later batch on the same game files with the same converter reuses them instead of converting again. When any mounted archive or the converter changes, the cache is cleared and rebuilt, so a stale model is never used. Packages are identical whether a model came from the cache or was converted.
- When a batch succeeds (every vehicle built or skipped as not paintable), its intermediate work folder under `target/game-extraction/` is deleted after the report is written. A batch with any failure keeps it, so the converter logs named in failure messages remain available.
- The report records whether the work folder was kept, and how many models were converted and how many were reused from the cache.

Non-goals: parallel model conversion, caching definitions or listings between batches, a `--no-cache` or `--keep-work` flag (deleting the cache folder by hand has the same effect), cleaning work folders left by earlier batches.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `ets2-game-extraction`: "Isolated batch outputs" removes the work folder of a successful batch and keeps it for an incomplete one; "Traceable technical report" records whether the work folder was kept and the model cache use; a new requirement covers the model cache shared between batches.

## Impact

- `tp-pack` extraction sources (`Sources::locate` and the inventory): a per-archive folder listing memo and top-down pruning. No new dependency, no change to the `tpv extract` arguments.
- `tp-pack` model conversion: convert into the batch, then move the result into the shared cache; validate the cache key at batch start.
- `tp-pack` batch run: delete the work folder after a successful batch; new report fields.
- Fake-converter tests: count converter calls, check the work folder is removed or kept, check a second batch converts nothing and a changed archive clears the cache.
- `docs/game-extraction-plan.md`: before/after call counts and duration of a full batch on the local game.
- Player documentation (four languages): the official vehicles page says intermediates and converter logs stay under `target/`; it must say they are kept only when a vehicle fails.
