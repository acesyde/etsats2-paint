## Why

Strokes are a single solid line of one width, centered on the edge. Liveries need more:
- **Outside** outlines around lettering, so the outline doesn't eat into thin letters. This is the most requested look for team names.
- **Inside** borders on panels.
- **Dashed** pinstripes and stitching lines.
- **Rounded or squared** line ends and corners.

All of these are standard in every vector editor and missing here.

## What Changes

- **Stroke alignment: Center** (today's behavior), **Inside** or **Outside** the edge, for shapes, paths and texts.
  - For lines (open paths), the outline sits inside or outside the line's body.
- **Dashes:**
  - a Dashed toggle with Dash and Gap lengths in texture pixels;
  - presets: Dashed (20/10), Dotted (0/12, with round caps), Long dash (60/20);
  - dashes follow curves and corners.
- **Caps:** Butt, Round or Square ends for open paths and dashes.
- **Joins:** Miter (with a miter limit), Round or Bevel corners.
- **Lines (open paths) have their own dash, cap and join**, next to Width in the Properties panel. A dashed pinstripe needs no outline, and a line's outline follows the line's dashes, caps and joins.
- **Stroke panel:** a compact second section for these options (for lines, the stroke panel's dash, cap and join don't apply: the line's own do). Every value applies to all selected strokes, shows "Mixed" when they differ, edits the current style with nothing selected, and is one undo step per change.
- **Rendering:** the canvas and the exported texture draw them identically.
- **Compatibility:**
  - existing projects look exactly the same: the defaults are centered, solid, round caps, and miter joins with limit 4, which is today's rendering;
  - the options are saved with the project.

Not in this change: variable-width strokes, arrowheads, several strokes on one object, gradient strokes, outlining a stroke into a path.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `stroke-panel`: adds the alignment, dash, cap and join controls.
- `document-model`: a stroke gains alignment, dash pattern, cap, join and miter limit, with defaults that keep the current rendering; lines get their own dash, cap and join.
- `path-tools`: the Properties panel's line settings gain Dashed / Dash / Gap, cap and join.

## Impact

- **`tp-core`:**
  - `StrokeStyle` gains `align`, `dash: Option<Dash>`, `cap`, `join` and `miter_limit` (still `Copy`); these style fields are grouped in a reusable `LineStyle` (dash, cap, join, miter limit) that `PathData` also carries for line bodies;
  - a `stroke_region` module computes the region to paint for a stroke: dashing (kurbo `dash`), expansion to an outline (kurbo `stroke`, with cap, join and miter limit), and clipping against the fill area for Inside and Outside;
  - the clipping uses the new dependency `i_overlay` 9.0.0 (MIT/Apache-2.0, polygon booleans) on paths flattened at the drawing tolerance.
- **`tp-render`:** strokes are drawn by filling their stroke region; lines keep the casing technique, adapted to alignment and dashes.
- **`tp-app`:**
  - the canvas meshes for rectangles and ellipses (currently egui polylines) move to the same stroke region when an option differs from the default;
  - text stroke meshes follow the stroke style;
  - the geometry cache key includes the stroke style;
  - the Stroke panel controls.
- **`tp-file`:** optional stroke fields in format 2 (unreleased, additive, with defaults). The v2 fixture is regenerated with a dashed outside stroke.
