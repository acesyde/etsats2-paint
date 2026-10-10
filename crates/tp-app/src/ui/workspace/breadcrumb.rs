//! The breadcrumb of the Workshop: `<vehicle> › Main textures|Accessories ›
//! <texture>` with the texture's size, or `Symbol › <name>` while a symbol
//! is edited. Shown in the top bar and inlaid on the canvas.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, CornerRadius, FontId, Label, Rect, Sense, Stroke, StrokeKind, Ui};
use tp_core::{Project, TexturePart};
use tp_i18n::tr;
use tp_ui::tokens::{color, radius, space, stroke, typography};

use super::canvas;
use crate::workspace::Workspace;

/// The parts of the breadcrumb.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crumbs {
    /// The groups before the name ("Vehicle", "Accessories"), in order.
    pub path: Vec<String>,
    /// What is painted: the texture or the symbol.
    pub name: String,
    /// The texture's size (`1024²`); none for a symbol.
    pub size: Option<String>,
}

impl Crumbs {
    pub fn of(ws: &Workspace) -> Self {
        if let Some(symbol) = ws.project.edited_symbol() {
            return Self {
                path: vec![tr("breadcrumb-symbol")],
                name: symbol.name.clone(),
                size: None,
            };
        }
        Self::of_texture(&ws.project, ws.project.active_surface)
    }

    /// The breadcrumb of texture `index`: "<vehicle> › Main textures|
    /// Accessories › <texture>" and its size (the command palette's texture
    /// rows too).
    pub fn of_texture(project: &Project, index: usize) -> Self {
        let surface = &project.surfaces[index];
        let size = Some(format!("{}²", surface.size.round() as u32));
        let Some((vehicle, texture)) = project.surface_names(index) else {
            return Self {
                path: Vec::new(),
                name: surface.name.clone(),
                size,
            };
        };
        let part = surface
            .template
            .as_ref()
            .map_or(TexturePart::Main, |t| t.part);
        let group = tr(match part {
            TexturePart::Main => "vehicles-main-textures",
            TexturePart::Accessory => "vehicles-accessories",
        });
        Self {
            path: vec![vehicle, group],
            name: texture,
            size,
        }
    }

    /// The whole breadcrumb as one line ("Truck › Accessories › Side skirts
    /// 1024²"): its accessible name and its hover text.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for part in &self.path {
            text.push_str(part);
            text.push_str(" › ");
        }
        text.push_str(&self.name);
        if let Some(size) = &self.size {
            text.push(' ');
            text.push_str(size);
        }
        text
    }

    /// The breadcrumb laid out on one line: the path dimmed, the name in
    /// ink, the size in the mono font. `compact` (the canvas pill) sets it
    /// a point smaller, the path and size muted.
    fn job(&self, ui: &Ui, compact: bool) -> LayoutJob {
        let mut body = egui::TextStyle::Body.resolve(ui.style());
        let mut strong = tp_ui::theme::label_strong_style().resolve(ui.style());
        let (dim, gap) = if compact {
            let scale = typography::CONTROL / typography::BODY;
            body.size *= scale;
            strong.size *= scale;
            (color::TEXT_MUTED, "  ")
        } else {
            (color::TEXT_SECONDARY, " ")
        };
        let mono = FontId::monospace(typography::CAPTION);
        let mut job = LayoutJob::default();
        let mut add = |text: &str, font: &FontId, color: Color32| {
            job.append(text, 0.0, TextFormat::simple(font.clone(), color));
        };
        for part in &self.path {
            add(part, &body, dim);
            add(" › ", &body, dim);
        }
        add(&self.name, &strong, color::TEXT_PRIMARY);
        if let Some(size) = &self.size {
            add(gap, &body, dim);
            add(size, &mono, dim);
        }
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job
    }
}

/// The breadcrumb as a label, shortened with an ellipsis when it does not
/// fit and shown in full on hover.
pub fn label(ui: &mut Ui, crumbs: &Crumbs) -> egui::Response {
    label_job(ui, crumbs, false)
}

fn label_job(ui: &mut Ui, crumbs: &Crumbs, compact: bool) -> egui::Response {
    let text = crumbs.text();
    let response = ui.add(
        Label::new(crumbs.job(ui, compact))
            .truncate()
            .selectable(false)
            .sense(Sense::hover()),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &text));
    response.on_hover_text(text)
}

/// The breadcrumb inlaid in the top left corner of the canvas area. It only
/// senses hover, so the canvas under it keeps every press and drag.
pub fn inlaid(ui: &mut Ui, ws: &Workspace) {
    let Some(area) = ws.canvas_rect else {
        return;
    };
    let crumbs = Crumbs::of(ws);
    // Under the bar of the symbol being edited.
    let top = if ws.is_editing_symbol() {
        canvas::SYMBOL_BAR_HEIGHT
    } else {
        0.0
    };
    let max = Rect::from_min_max(
        area.min + egui::vec2(space::MD, top + space::SM + 2.0),
        egui::pos2(area.right() - space::MD, area.bottom()),
    );
    if max.width() < 40.0 || max.height() < 24.0 {
        return;
    }
    let pad = egui::vec2(space::SM + 2.0, space::XS);
    let galley = ui.painter().layout_job(crumbs.job(ui, true));
    let width = (galley.size().x + 2.0 * pad.x).min(max.width());
    let chip = Rect::from_min_size(max.min, egui::vec2(width, galley.size().y + 2.0 * pad.y));
    ui.painter().rect(
        chip,
        CornerRadius::same(radius::LG),
        color::CONTROL,
        Stroke::new(stroke::HAIRLINE, color::BORDER),
        StrokeKind::Inside,
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(chip.shrink2(pad))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(chip.intersect(ui.clip_rect()));
    label_job(&mut child, &crumbs, true);
}
