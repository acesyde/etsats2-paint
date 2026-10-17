## Why

`tpv extract` now builds all 26 official ETS2 trucks from the installed game, but players cannot get official trailers any more: the retired template-pack catalog was the only source, and the extractor deliberately refuses trailers. The installed game defines 58 owned trailers (`def/vehicle/trailer_owned/`), against 39 in Paintjob Packer's data, so the same game-driven pipeline can cover trailers more completely than the community source did.

## What Changes

- `tpv extract` discovers and extracts official ETS2 owned trailers (`def/vehicle/trailer_owned/<brand>.<series>/`), including trailer DLC, alongside trucks. Without `--vehicle` a batch processes every discovered truck and trailer; `--vehicle krone.coolliner` processes one trailer.
- Trailer packages reuse the truck pipeline unchanged: archive sources and precedence, ConverterPIX conversion, look/variant/visible-part selection, ordinary `_TEXCOORD1` UVs, crop-size rules (`resolved_largest`, `raised_to_minimum`, `main_texture`), exclusion of accessories without livery material, game names, `--previous` layout versions, isolated batches and the JSON report.
- A trailer package has `kind: "trailer"`, exactly one main texture (4096 × 4096, no cabin ids) covering every painted part without its own sized mask, and accessory textures shared by the whole trailer. Trailers have no variants.
- Accessory ids that use the same sized mask in every game paint job that overrides them become **one** accessory texture with several game ids, whose template superposes their layouts (as the game paints them from one region, and as Paintjob Packer groups them). Truck packages are unchanged by this rule.
- Trailer names come from the game's localization like trucks. A trailer with an empty brand (SCS's own trailers) is named "SCS" followed by its game series id in readable form ("SCS Box", "SCS Flatbed"), because the game's series strings for them are abbreviations ("STD", "FLB").
- The report lists trucks and trailers together, each with its kind, and a comparison note against Paintjob Packer is recorded in the extraction source notes after the first full batch.

Non-goals: ATS trailers, truck-and-trailer combination presets (B-double and double configurations), trailer variants, grouping identical masks for trucks, a `--kind` filter, in-game validation of every trailer.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `ets2-game-extraction`: discovery and the `--vehicle` filter cover owned trailers; trailer naming; trailer package shape (one main texture, shared accessories, accessory grouping by shared mask); truck/trailer id collisions refused.
- `official-vehicles`: generated official ETS2 trailers use `official.ets2.<trailer>` ids and the same manual installation workflow as trucks.

## Impact

- `tp-pack` extraction module: discovery over `def/vehicle/trailer_owned/`, a trailer build path reusing the truck stages, mask-based accessory grouping, trailer naming; fake-converter tests for trailers. No new dependency.
- No application change expected: the app already installs `kind: "trailer"` packages and exports trailer paint jobs under `trailer_owned` (existing Krone Cool Liner export test).
- Documentation (four languages): the official vehicles page covers trailers; `docs/game-extraction-plan.md` records the first full batch and the Paintjob Packer comparison.
- A full batch roughly triples in length (26 trucks + 58 trailers).
