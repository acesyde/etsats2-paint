//! The dialog another dialog was opened from (Custom vehicle…, the template
//! pack dialog), put back as it was when that one closes.

use tp_vehicles::{Game, Manifest};

use super::vehicle_dialogs::{
    AddVehicleDialog, LibraryDialog, VehicleChoice, VehicleFilter, push_choice,
};
use crate::official::InstallJob;
use crate::state::{Modal, NewProjectDraft};
use crate::vehicles::VehicleLibrary;

/// The dialog a dialog was opened from.
pub enum Origin {
    NewProject(NewProjectDraft),
    AddVehicle(AddVehicleDialog),
    Library(LibraryDialog),
}

impl Origin {
    pub(crate) fn into_modal(self) -> Modal {
        match self {
            Origin::NewProject(draft) => Modal::NewProject(draft),
            Origin::AddVehicle(dialog) => Modal::AddVehicle(dialog),
            Origin::Library(dialog) => Modal::VehicleLibrary(dialog),
        }
    }

    /// The game the origin shows, if any.
    pub(crate) fn game(&self) -> Option<Game> {
        match self {
            Origin::NewProject(draft) => draft.filter.game,
            Origin::AddVehicle(dialog) => dialog.filter.game,
            Origin::Library(dialog) => dialog.filter.game,
        }
    }

    /// Shows `m`, just installed: checked in New Project (its card open) and
    /// Add Vehicle, after the vehicles checked before, listed in the
    /// library, with a confirmation.
    pub(crate) fn created(&mut self, library: &VehicleLibrary, m: &Manifest, message: String) {
        let reveal = |filter: &mut VehicleFilter| {
            if !filter.matches(m) {
                filter.query.clear();
                filter.kind = None;
                if !filter.game_locked {
                    filter.game = None;
                }
            }
        };
        let choice = library.get(&m.id).map(VehicleChoice::of);
        match self {
            Origin::NewProject(draft) => {
                // The dialog lists one game: the vehicle's (the vehicles of
                // the other game are unchecked).
                draft.set_game(m.game.id, library);
                reveal(&mut draft.filter);
                draft.messages = vec![Ok(message)];
                if let Some(choice) = choice {
                    draft.check(choice);
                }
            }
            Origin::AddVehicle(dialog) => {
                reveal(&mut dialog.filter);
                dialog.messages = vec![Ok(message)];
                if let Some(choice) = choice {
                    push_choice(&mut dialog.choices, choice);
                }
            }
            Origin::Library(dialog) => {
                reveal(&mut dialog.filter);
                dialog.messages = vec![Ok(message)];
            }
        }
    }

    /// Checks the installed vehicle `choice` in New Project (opening its
    /// card) or Add Vehicle…, after the vehicles checked before.
    pub(crate) fn choose(&mut self, choice: VehicleChoice) {
        match self {
            Origin::NewProject(draft) => {
                draft.messages.clear();
                draft.check(choice);
            }
            Origin::AddVehicle(dialog) => {
                dialog.messages.clear();
                push_choice(&mut dialog.choices, choice);
            }
            Origin::Library(_) => {}
        }
    }

    /// Shows `job` installing an official vehicle in New Project or Add
    /// Vehicle (the library doesn't pick vehicles).
    pub(crate) fn start_install(&mut self, job: InstallJob) {
        let job = Some(std::sync::Arc::new(job));
        match self {
            Origin::NewProject(draft) => {
                draft.messages.clear();
                draft.installing = job;
            }
            Origin::AddVehicle(dialog) => {
                dialog.messages.clear();
                dialog.installing = job;
            }
            Origin::Library(_) => {}
        }
    }
}
