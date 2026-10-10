//! eframe integration.

use crate::state::AppState;

pub struct TruckPaintApp {
    pub state: AppState,
    /// The menus in the macOS menu bar.
    #[cfg(target_os = "macos")]
    native_menu: crate::native_menu::NativeMenu,
    /// The traffic lights are centered in the top bar (first frame, once
    /// the window exists).
    #[cfg(target_os = "macos")]
    lights_centered: bool,
}

impl TruckPaintApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        mut state: AppState,
        recovery_dir: Option<std::path::PathBuf>,
        vehicles_dir: Option<std::path::PathBuf>,
        library_path: Option<std::path::PathBuf>,
    ) -> Self {
        state.library = crate::library::LibraryStore::new(library_path);
        if let Some(dir) = vehicles_dir {
            state.vehicles = crate::vehicles::VehicleLibrary::open(&dir);
        }
        state.system_language = sys_locale::get_locale()
            .map(|locale| tp_i18n::Language::from_locale(&locale))
            .unwrap_or_default();
        tracing::info!(language = state.system_language.code(), "system language");
        if let Some(dir) = recovery_dir {
            state.enable_recovery(&dir);
        }
        state.install_theme(&cc.egui_ctx);
        #[cfg(target_os = "macos")]
        let native_menu = {
            // Built in the interface language, before the first frame sets it.
            tp_i18n::set_language(state.language());
            let menu = crate::native_menu::NativeMenu::install(&cc.egui_ctx, state.language());
            state.native_menu_installed = true;
            menu
        };
        Self {
            state,
            #[cfg(target_os = "macos")]
            native_menu,
            #[cfg(target_os = "macos")]
            lights_centered: false,
        }
    }
}

impl eframe::App for TruckPaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "macos")]
        if !self.lights_centered {
            crate::title_bar::center_traffic_lights();
            self.lights_centered = true;
        }
        self.state.show(ui);
        #[cfg(target_os = "macos")]
        self.native_menu.sync(ui.ctx(), &self.state);
    }

    /// The macOS menu bar's clicks and keys join the frame's input.
    #[cfg(target_os = "macos")]
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.native_menu
            .take_input(raw_input, &mut self.state.queue);
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
