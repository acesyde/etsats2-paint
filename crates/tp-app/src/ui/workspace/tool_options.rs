//! The tool options bar under the top bar of the Workshop: the active tool's
//! name and, for the Polygon tool, its settings (Sides, Star, Inner radius).

use egui::{RichText, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};

use super::panels::{PanelEnv, properties};
use crate::tool::Tool;

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    ui.horizontal_centered(|ui| {
        let tool = env.ws.tool;
        let name = tr(tool.name());
        ui.label(
            RichText::new(&name)
                .text_style(label_strong_style())
                .color(color::TEXT_PRIMARY),
        )
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &name));
        if tool == Tool::Polygon {
            ui.add_space(space::SM);
            ui.separator();
            ui.add_space(space::SM);
            properties::polygon_settings(ui, env);
        }
    });
}
