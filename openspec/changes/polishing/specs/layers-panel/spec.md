## MODIFIED Requirements

### Requirement: Tree view
The Layers panel SHALL be the **Layers** tab of the Workshop's left panel (key 2, see the workspace-layout capability). The tab SHALL have a header reading "<texture> · N layers", where N is the number of top-level objects of the active surface, with an **add** button that runs New Layer. Below it, the tab SHALL list the active surface's objects as a tree, topmost first, with groups expandable and collapsible and their children indented. Each row SHALL show its kind icon on a tile, the name, its kind in small type (left out when the name is the kind's), a visibility toggle and a lock toggle. A name is the kind's when it is the kind's default name in any of the supported languages ("Text", "Texte", "Texto"), so that a document made in one language reads the same in another. A text layer whose name is the kind's SHALL show the first non-empty line of its text instead, with its kind in small type. A name too long for its row SHALL end with "…" and show in full in a tooltip, never be cut mid-letter; names are in the secondary text color and in ink when selected; hidden and locked states SHALL be shown by icon shape and by dimming the name, not only by color. An empty surface SHALL show an empty state in the list, under the header.

#### Scenario: Group shown as a tree
- **WHEN** the surface contains a group "Branding" with "Logo" and "Company name", above a rectangle "Background"
- **THEN** the tab shows "Branding" first with its two children indented below it, then "Background"

#### Scenario: Tab header
- **WHEN** the active texture "Side skirts" has five top-level objects and the user opens the Layers tab
- **THEN** the header reads "Side skirts · 5 layers", and clicking its add button adds an empty group "Layer N" at the top of the tree

#### Scenario: Text layers named by their text
- **WHEN** the surface holds two texts named "Text", reading "ACE LOGISTICS" and "Since 1984"
- **THEN** their rows read "ACE LOGISTICS" and "Since 1984", each with the kind "Text" in small type

#### Scenario: Default name from another language
- **WHEN** a rectangle was named "Rectangle" in English and the language is now German
- **THEN** its row reads "Rectangle" once, without the kind "Rechteck" next to it

#### Scenario: Long layer name
- **WHEN** a layer is named "Grey stripe along the lower edge of the cabin" and the panel is at its default width
- **THEN** its row shows the start of the name ending with "…", and resting the pointer on it shows the whole name
