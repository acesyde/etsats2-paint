## Context

See proposal.md for motivation. The extractor (`crates/tp-pack/src/extract/`) is built around trucks:

- `Sources::inventory` extracts only `/def/vehicle/truck` from every archive; discovery keeps `def/vehicle/truck/<id>/data.sii`.
- `Build` roots everything at `/def/vehicle/truck/<id>/`, reads identity from `accessory_truck_data`, walks `cabin/`, `chassis/` and `accessory/` units, and builds one 4096² main texture per cabin from compatible contributors.
- Paint overrides (`paint_job/accessory/*.sii`) are collected per accessory id as `mask → definitions`; size rules, exclusions, `--previous` layout versions and the report are vehicle-agnostic.

Owned trailers, observed in ETS2 1.61.1.9 (`scs.box`, `krone.coolliner`):

- `def/vehicle/trailer_owned/<id>/data.sii` holds `accessory_trailer_data : vehicle.<id>` with `info[]` brand (empty for SCS trailers) and series.
- Painted units live in `body/` (`accessory_trailer_body_data : curtain_136.scs.box.body`), `chassis/` and `accessory/`; unit ids follow the truck `<name>.<id>.<group>` pattern, so override ids (`body.curtain_136`, `chs_f_l.…`, `r_bumper.…`) match the existing `group.name` convention.
- Every paint job's main mask is a 4×4 fill (`pjm_at_0x0_size_4x4`); liveries live in overrides with sized crops of a large atlas (`pjm_at_4096x6144_size_4096x2048`), and several ids share one crop (dry van, insulated and reefer 13.6 m bodies).
- The app already installs `kind: "trailer"` packages with one main texture and multi-id accessories, and its mod export writes `trailer_owned` paint jobs with one override per accessory texture listing all its game ids.

## Goals / Non-Goals

**Goals:**
- One build pipeline for trucks and trailers; the truck path and truck packages stay byte-identical.
- Mask-shared grouping defined purely from game data, testable with the fake converter.

**Non-Goals:**
- Grouping for trucks (would change published truck packages; a separate decision).
- Combination presets (`configurations/` B-double, double) and per-trailer profiles.

## Decisions

1. **Vehicle kind as data, not a second pipeline.** A small `Kind { Truck, Trailer }` value carries the definition root (`truck` / `trailer_owned`), the data unit header (`accessory_truck_data` / `accessory_trailer_data`), the painted groups to walk (`cabin/chassis/accessory` vs `body/chassis/accessory`) and the manifest `kind`. Alternative rejected: a separate trailer builder, which would duplicate geometry, size, exclusion and report code that is already kind-agnostic.

2. **Discovery and inventory.** `inventory` extracts both `/def/vehicle/truck` and `/def/vehicle/trailer_owned`; discovery collects ids from both `data.sii` sets. An id present in both is reported and refused (spec: no silent choice). `--vehicle` resolves against the union.

3. **Single trailer main texture.** For trailers the "cabin loop" becomes one main texture with id `trailer`, name "Trailer", no game ids, 4096², whose contributors are all painted parts without an independent size (bodies, chassis, accessories alike; `suitable_for` is ignored since there is one main texture). The 4×4 main mask carries no size, so the truck cabin size rule (4096²) applies; it is the same main-texture size trucks use and keeps one rule.

   When every painted part has its own region (most branded trailers: bodies and chassis are all overridden), nothing contributes to the main texture and the game's main mask is a 4×4 fill: the main texture is then 64×64 with a blank SVG template, carrying only the base colour. Trailers whose paint jobs only set (often locked) plain colours, with no mask and no override, are reported as `skipped` / `not_paintable` instead of failures: no livery is possible for them in the game (SCS flatbed, lowbed, log…), and counting them as failures would make every batch fail. A definition with no model field at all (Krone Box Liner's container body) is excluded and reported; an existing but unreadable model still blocks.

4. **Grouping by identical override signature.** For each accessory id with sized masks, its *signature* is the set of `(paint job definition, mask)` pairs that override it. Ids with equal signatures form one accessory texture: game ids sorted, part id from the first id, name listing the shared mask's body/part names, template = union of their meshes, size = size rules applied to the shared mask set. Ids whose signatures differ stay separate, which covers "differ in at least one paint job" without special cases. This runs only for trailers. Alternative rejected: grouping by mask path alone, which would merge ids that some paint jobs paint differently.

5. **SCS trailer names.** When the brand label is empty, the name is `SCS ` + the series part of the game id (`box` → `Box`, `fueltank` → `Fueltank`), words capitalized on `_`. Branded trailers use the existing brand + series rule unchanged.

6. **Batch size.** A full batch processes trucks then trailers in id order, as today; converted models are already cached per batch, and shared chassis models across trailers convert once. No new flag; `--vehicle` stays the way to run a subset.

## Risks / Trade-offs

- [Trailer models may use painted materials or UV streams the truck rules refuse] → They are refused per trailer with the existing diagnostics; the first full batch report lists them, and rules are only relaxed in a follow-up with evidence, as was done for trucks.
- [Unions of many bodies/chassis in one 4096² main template can be dense] → Accepted: it mirrors the game, where those parts share the main mask; the report lists contributors per texture.
- [A grouped texture's name may be long when many ids share it] → Names list at most the distinct body/part names; the game ids stay complete in the manifest.
- [Full batches get about three times longer] → Documented; `--vehicle` remains available.

## Migration Plan

No migration: trailer packages are new ids. Truck packages must stay byte-identical, verified by comparing a truck batch with the previous one (`--previous` keeps layout versions, and bytes are compared directly).

## Open Questions

- Which trailers need in-game checks first is decided from the first full batch report (unusual UV bounds, grouped textures, refusals).
