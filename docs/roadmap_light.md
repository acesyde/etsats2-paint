# Roadmap notes (working draft)

Short notes from the discussion of the open questions after `mod-export`.
Decisions move to [roadmap.md](roadmap.md) once they become changes.

## Decided

- **Remove the 3D preview** (done in `remove-3d-preview`). It has only ever
  been a placeholder. What goes:
  - the view modes 2D / 3D / Split (segmented control, View menu, shortcuts);
  - Toggle Preview and `ui/workspace/preview.rs`;
  - the layout fields in the preferences;
  - the requirements "3D preview panel placeholder" and "View modes" in
    `workspace-layout`, and the mentions in `app-preferences` and
    `document-model`.

  Trap: a preferences file holding `view_mode: ThreeD` must still load (read
  the old value and ignore it). Otherwise it fails to parse and every
  preference is reset to defaults.
- **No copy of a design to the other side of the truck** (recorded in the
  roadmap with `game-versions`). Copy/paste and Flip cover it.
- **Prefill from Paintjob Packer's database:** later.

## Game versions (done in `game-versions`)

Decided differently from the proposal below: the project's Game versions
are an editable list copied as is into `compatible_versions[]`
(`1.56.*, 1.57.*`), checked against the packages' ranges; a fleet with no
common version blocks the export. The proposal, for the record:

Packages give version ranges (`>=1.56`, `^1.53`, `*`). The manifest's
`compatible_versions[]` wants explicit patterns (`"1.56.*"`).

```
  Sample Truck    >=1.56 ----------------------------->
  Sample Trailer  >=1.50 ----------------------------->
  Volvo FH (mod)          >=1.53, <1.58 ------|
                                 |
                      intersection: >=1.56, <1.58
```

Proposal:
- the Project section's **Game versions** shows the intersection of the
  fleet's ranges, computed and read only;
- **no `compatible_versions`** in the manifest. Open ranges can't become a
  list without knowing the current game version. A list would also make the
  Mod Manager flag the mod after every game update, even when the textures
  didn't change, until the player exports again;
- an empty intersection (two vehicles that never run on the same game
  version) is reported in Export Mod. Warning or blocking problem: to decide.

## Shared library (to decide)

Reuse a logo, swatches and styles across projects, including across ETS2
and ATS.

| | What | Cost |
|---|---|---|
| **A. Import from a project** | "Import from project…": pick a `.truckpaint`, check symbols, swatches and styles, copy them in. The library is simply a project. | Small |
| **B. Personal library** | A Library panel stored in the app's data folder. "Add to library" from a project, "Place" / "Import" into another. Copied, not linked. | Medium |
| **C. Linked library** | Like B, but projects follow the library's edits (update prompts, versions). | Large |

Leaning: **B**.

## Bug found: pasting into another project

The clipboard (`AppState.clipboard: Vec<Object>`) survives closing a
project, and its objects refer to ids of the source project:

```
 Project A                         Project B (paste)
 +------------------+              +------------------+
 | Image -> asset#7 |  -- copy --> | asset#7 = ???    |  missing or another image
 | Fill -> swatch#3 |              | swatch#3 = ???   |  link to the wrong swatch
 | Instance -> sym#5|              | sym#5 = ???      |  instance without symbol
 +------------------+              +------------------+
```

Any library needs the same building block: bringing an element into a
project with everything it depends on (assets, swatches, styles, symbol)
under new ids. Fixing paste comes with it.

## Possible changes

1. A small cleanup change: remove the 3D preview, Game versions, roadmap
   updates.
2. The library change (with the paste fix), once A, B or C is chosen.
