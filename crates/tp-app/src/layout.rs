//! Workspace layout model (panels, view mode), independent of rendering so it
//! can be persisted and unit-tested.

use serde::{Deserialize, Serialize};
use tp_ui::{icons, tokens::size};

/// The panels of the right-hand column, in default order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelKind {
    Properties,
    Layers,
    Colors,
    Styles,
    Stroke,
    Transform,
    Assets,
    /// Former panel of the column, now the Vehicles sidebar on the left;
    /// kept so preferences saved by earlier builds still load (it is
    /// dropped from the column when they are read).
    Vehicle,
}

impl PanelKind {
    /// The panels of the right-hand column.
    pub const ALL: [Self; 7] = [
        Self::Properties,
        Self::Layers,
        Self::Colors,
        Self::Styles,
        Self::Stroke,
        Self::Transform,
        Self::Assets,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Properties => "panel-properties",
            Self::Layers => "panel-layers",
            Self::Colors => "panel-colors",
            Self::Styles => "panel-styles",
            Self::Stroke => "panel-stroke",
            Self::Transform => "panel-transform",
            Self::Assets => "panel-assets",
            Self::Vehicle => "panel-vehicle",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Properties => icons::PROPERTIES,
            Self::Layers => icons::LAYERS,
            Self::Colors => icons::COLORS,
            Self::Styles => icons::STYLES,
            Self::Stroke => icons::STROKE,
            Self::Transform => icons::TRANSFORM,
            Self::Assets => icons::ASSETS,
            Self::Vehicle => icons::VEHICLE,
        }
    }

    /// Empty-state text shown until the panel's feature exists.
    pub fn empty_state(self) -> (&'static str, &'static str) {
        match self {
            Self::Properties => ("empty-properties", "empty-properties-hint"),
            Self::Layers => ("empty-layers", "empty-layers-hint"),
            Self::Colors => ("empty-colors", "empty-colors-hint"),
            Self::Styles => ("empty-styles", "empty-styles-hint"),
            Self::Stroke => ("empty-stroke", "empty-stroke-hint"),
            Self::Transform => ("empty-transform", "empty-transform-hint"),
            Self::Assets => ("empty-assets", "empty-assets-hint"),
            Self::Vehicle => ("empty-vehicle", "empty-vehicle-hint"),
        }
    }
}

/// How the central area is split between the 2D canvas and the 3D preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    TwoD,
    ThreeD,
    Split,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::TwoD => "view-2d",
            Self::ThreeD => "view-3d",
            Self::Split => "view-split",
        }
    }

    pub fn shows_canvas(self) -> bool {
        matches!(self, Self::TwoD | Self::Split)
    }

    pub fn shows_preview(self) -> bool {
        matches!(self, Self::ThreeD | Self::Split)
    }
}

/// A panel's place and state in the column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelSlot {
    pub kind: PanelKind,
    pub open: bool,
    pub collapsed: bool,
}

pub const SPLIT_FRACTION_DEFAULT: f32 = 0.42;
/// Width of the Vehicles sidebar.
pub const VEHICLES_WIDTH_DEFAULT: f32 = 260.0;
pub const VEHICLES_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 200.0..=420.0;
pub const SPLIT_FRACTION_RANGE: std::ops::RangeInclusive<f32> = 0.2..=0.8;

/// Everything about the workspace arrangement that survives restarts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceLayout {
    /// Panels in display order; closed panels keep their position.
    pub panels: Vec<PanelSlot>,
    pub column_width: f32,
    pub view_mode: ViewMode,
    /// Width of the 3D preview in split view, as a fraction of the central area.
    pub split_fraction: f32,
    /// The Vehicles sidebar on the left: shown, or reduced to a strip.
    pub vehicles_open: bool,
    pub vehicles_width: f32,
    /// Bumped on reset so egui forgets remembered panel sizes.
    #[serde(skip)]
    pub generation: u32,
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self {
            panels: PanelKind::ALL
                .iter()
                .map(|&kind| PanelSlot {
                    kind,
                    open: true,
                    // Keep the default stack readable: secondary panels start collapsed.
                    collapsed: matches!(
                        kind,
                        PanelKind::Styles
                            | PanelKind::Stroke
                            | PanelKind::Transform
                            | PanelKind::Assets
                    ),
                })
                .collect(),
            column_width: size::PANEL_COLUMN_DEFAULT,
            view_mode: ViewMode::default(),
            split_fraction: SPLIT_FRACTION_DEFAULT,
            vehicles_open: true,
            vehicles_width: VEHICLES_WIDTH_DEFAULT,
            generation: 0,
        }
    }
}

impl WorkspaceLayout {
    /// Repairs a deserialized layout: every panel exactly once, values in range.
    pub fn sanitized(mut self) -> Self {
        let mut seen = Vec::new();
        self.panels.retain(|slot| {
            let fresh = !seen.contains(&slot.kind) && PanelKind::ALL.contains(&slot.kind);
            seen.push(slot.kind);
            fresh
        });
        // A panel the saved layout doesn't know (it was added since) goes
        // to its default place, after the panel before it, collapsed.
        for (i, kind) in PanelKind::ALL.into_iter().enumerate() {
            if !seen.contains(&kind) {
                let at = PanelKind::ALL[..i]
                    .iter()
                    .rev()
                    .find_map(|prev| self.panels.iter().position(|s| s.kind == *prev))
                    .map_or(0, |p| p + 1);
                self.panels.insert(
                    at,
                    PanelSlot {
                        kind,
                        open: true,
                        collapsed: true,
                    },
                );
            }
        }
        self.column_width = self
            .column_width
            .clamp(size::PANEL_COLUMN_MIN, size::PANEL_COLUMN_MAX);
        self.split_fraction = self
            .split_fraction
            .clamp(*SPLIT_FRACTION_RANGE.start(), *SPLIT_FRACTION_RANGE.end());
        self.vehicles_width = self
            .vehicles_width
            .clamp(*VEHICLES_WIDTH_RANGE.start(), *VEHICLES_WIDTH_RANGE.end());
        self
    }

    pub fn slot(&self, kind: PanelKind) -> &PanelSlot {
        self.panels
            .iter()
            .find(|slot| slot.kind == kind)
            .expect("sanitized layout contains every panel")
    }

    pub fn slot_mut(&mut self, kind: PanelKind) -> &mut PanelSlot {
        self.panels
            .iter_mut()
            .find(|slot| slot.kind == kind)
            .expect("sanitized layout contains every panel")
    }

    pub fn is_open(&self, kind: PanelKind) -> bool {
        self.slot(kind).open
    }

    /// Opens a closed panel (expanded) or closes an open one.
    pub fn toggle_open(&mut self, kind: PanelKind) {
        let slot = self.slot_mut(kind);
        slot.open = !slot.open;
        if slot.open {
            slot.collapsed = false;
        }
    }

    pub fn toggle_collapsed(&mut self, kind: PanelKind) {
        let slot = self.slot_mut(kind);
        slot.collapsed = !slot.collapsed;
    }

    /// Restores defaults (all panels open, default order and sizes).
    pub fn reset(&mut self) {
        let generation = self.generation.wrapping_add(1);
        *self = Self {
            panels: PanelKind::ALL
                .iter()
                .map(|&kind| PanelSlot {
                    kind,
                    open: true,
                    collapsed: false,
                })
                .collect(),
            generation,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_keeps_position_and_reopening_expands() {
        let mut layout = WorkspaceLayout::default();
        let index = layout
            .panels
            .iter()
            .position(|s| s.kind == PanelKind::Colors)
            .unwrap();
        layout.toggle_collapsed(PanelKind::Colors);
        layout.toggle_open(PanelKind::Colors);
        assert!(!layout.is_open(PanelKind::Colors));
        layout.toggle_open(PanelKind::Colors);
        assert!(layout.is_open(PanelKind::Colors));
        assert!(!layout.slot(PanelKind::Colors).collapsed);
        assert_eq!(layout.panels[index].kind, PanelKind::Colors);
    }

    #[test]
    fn sanitize_repairs_missing_and_duplicate_panels() {
        let layout = WorkspaceLayout {
            panels: vec![
                PanelSlot {
                    kind: PanelKind::Layers,
                    open: false,
                    collapsed: false,
                },
                PanelSlot {
                    kind: PanelKind::Layers,
                    open: true,
                    collapsed: true,
                },
                // The former Vehicle panel of earlier builds.
                PanelSlot {
                    kind: PanelKind::Vehicle,
                    open: true,
                    collapsed: false,
                },
            ],
            column_width: 10_000.0,
            split_fraction: -1.0,
            vehicles_width: 5.0,
            ..WorkspaceLayout::default()
        }
        .sanitized();
        assert_eq!(layout.panels.len(), PanelKind::ALL.len());
        assert!(layout.panels.iter().all(|s| s.kind != PanelKind::Vehicle));
        assert_eq!(layout.vehicles_width, *VEHICLES_WIDTH_RANGE.start());
        assert!(layout.vehicles_open);
        assert!(!layout.is_open(PanelKind::Layers));
        assert_eq!(layout.column_width, size::PANEL_COLUMN_MAX);
        assert_eq!(layout.split_fraction, *SPLIT_FRACTION_RANGE.start());
    }

    #[test]
    fn layout_from_before_the_styles_panel() {
        let mut saved = WorkspaceLayout::default();
        saved.panels.retain(|s| s.kind != PanelKind::Styles);
        saved.panels[2].collapsed = true; // Colors
        saved.panels[3].open = false; // Stroke
        let layout = saved.clone().sanitized();
        let kinds: Vec<PanelKind> = layout.panels.iter().map(|s| s.kind).collect();
        assert_eq!(kinds, PanelKind::ALL);
        let styles = layout.slot(PanelKind::Styles);
        assert!(styles.open && styles.collapsed);
        assert!(layout.slot(PanelKind::Colors).collapsed);
        assert!(!layout.is_open(PanelKind::Stroke));
    }

    #[test]
    fn reset_reopens_everything_and_bumps_generation() {
        let mut layout = WorkspaceLayout::default();
        layout.toggle_open(PanelKind::Assets);
        layout.column_width = 400.0;
        layout.vehicles_open = false;
        layout.vehicles_width = 400.0;
        layout.reset();
        assert!(layout.panels.iter().all(|s| s.open && !s.collapsed));
        assert_eq!(layout.column_width, size::PANEL_COLUMN_DEFAULT);
        assert!(layout.vehicles_open);
        assert_eq!(layout.vehicles_width, VEHICLES_WIDTH_DEFAULT);
        assert_eq!(layout.generation, 1);
    }
}
