## ADDED Requirements

### Requirement: Surface guides
Each surface SHALL hold an ordered list of guides. A guide is horizontal or vertical and has a position in texture pixels (y for horizontal guides, x for vertical guides); positions outside the surface are allowed. A new surface SHALL have no guides.

#### Scenario: New project has no guides
- **WHEN** a project is created
- **THEN** its surface has no guides
