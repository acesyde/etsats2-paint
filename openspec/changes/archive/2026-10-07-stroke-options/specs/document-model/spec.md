## ADDED Requirements

### Requirement: Stroke style
A stroke SHALL have, besides its color and width:

- an **alignment**: Center (straddles the edge, default), Inside (within the filled area) or Outside (outside it);
- an optional **dash pattern**: a dash length (0 or more) and a gap length (more than 0), in texture pixels, starting at the beginning of each subpath;
- a **cap**: Butt, Round (default) or Square, which applies to the ends of open paths and of each dash;
- a **join**: Miter (default) with a miter limit (default 4), Round or Bevel.

The defaults SHALL render exactly like strokes did before these options existed. The stroke width SHALL be the visible thickness whatever the alignment.

Alignment by kind:
- **Shapes and texts:** Inside and Outside SHALL be measured against the object's filled area, including holes. An Inside stroke never paints outside the filled area, and an Outside stroke never paints inside it.
- **Lines (open paths):** Inside puts the outline within the line's body, and Outside puts it around the body.

Lines have their own dash pattern, cap and join (defaults: solid, round caps, miter joins with limit 4), applied to the line's body. A line's stroke (outline) follows the line's dashes, caps and joins; the stroke's own dash, cap and join SHALL NOT apply to lines.

#### Scenario: Outside outline on lettering
- **WHEN** a text with a 20 px Outside stroke is drawn
- **THEN** the letters keep their full shape and the outline is 20 px wide around them, including around the inner edge of the "O"'s counter

#### Scenario: Inside border
- **WHEN** a 400 px square has a 30 px Inside stroke
- **THEN** its outer edge is still at 400 px and the border covers the outer 30 px of the square

#### Scenario: Dotted line
- **WHEN** a 200 px line has a 10 px stroke with dash 0, gap 20 and round caps
- **THEN** it is drawn as round dots 20 px apart

#### Scenario: Dashed pinstripe without outline
- **WHEN** a red 12 px line without stroke is set to Dashed 40/20
- **THEN** it is drawn as red dashes 40 px long separated by 20 px gaps

#### Scenario: Old projects unchanged
- **WHEN** a project saved before stroke options existed is opened
- **THEN** every stroke is drawn exactly as before (centered, solid, round caps, miter joins)
