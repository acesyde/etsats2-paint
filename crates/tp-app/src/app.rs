//! eframe integration.

use crate::prefs::PrefsStore;
use crate::state::AppState;

pub struct TruckPaintApp {
    pub state: AppState,
}

impl TruckPaintApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        store: Option<PrefsStore>,
        recovery_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let mut state = AppState::new(store);
        state.system_language = sys_locale::get_locale()
            .map(|locale| tp_i18n::Language::from_locale(&locale))
            .unwrap_or_default();
        tracing::info!(language = state.system_language.code(), "system language");
        if let Some(dir) = recovery_dir {
            state.enable_recovery(&dir);
        }
        state.install_theme(&cc.egui_ctx);
        Self { state }
    }
}

impl eframe::App for TruckPaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.state.show(ui);
    }

    fn on_exit(&mut self) {
        self.state.shutdown();
        self.state.persist_prefs(f64::MAX, true);
    }

    fn persist_egui_memory(&self) -> bool {
        // Layout is stored in our own preferences; egui memory would only
        // restore stale panel sizes.
        false
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(tp_ui::tokens::color::SURFACE_0).to_array()
    }
}
