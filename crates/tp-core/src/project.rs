use serde::{Deserialize, Serialize};

/// Name given to a project created without a name.
pub const DEFAULT_PROJECT_NAME: &str = "Untitled";

/// Square texture resolution a livery is authored for.
///
/// The document itself stays vector-based; the resolution is the nominal
/// texture size used for coordinates and default export.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureResolution {
    R2048,
    #[default]
    R4096,
    R8192,
}

impl TextureResolution {
    pub const ALL: [Self; 3] = [Self::R2048, Self::R4096, Self::R8192];

    /// Side length in pixels.
    pub fn side(self) -> u32 {
        match self {
            Self::R2048 => 2048,
            Self::R4096 => 4096,
            Self::R8192 => 8192,
        }
    }

    /// Human-readable label, e.g. `4096 × 4096`.
    pub fn label(self) -> String {
        let side = self.side();
        format!("{side} × {side}")
    }
}

/// Minimal project description used until the full document model exists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectStub {
    pub name: String,
    pub resolution: TextureResolution,
}

impl ProjectStub {
    /// Creates a project, trimming the name and falling back to
    /// [`DEFAULT_PROJECT_NAME`] when it is empty.
    pub fn new(name: &str, resolution: TextureResolution) -> Self {
        let trimmed = name.trim();
        let name = if trimmed.is_empty() {
            DEFAULT_PROJECT_NAME.to_owned()
        } else {
            trimmed.to_owned()
        };
        Self { name, resolution }
    }

    /// Texture size in pixels (width, height).
    pub fn texture_size(&self) -> (u32, u32) {
        let side = self.resolution.side();
        (side, side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_resolution_is_4096() {
        assert_eq!(TextureResolution::default().side(), 4096);
    }

    #[test]
    fn resolution_labels() {
        assert_eq!(TextureResolution::R2048.label(), "2048 × 2048");
        assert_eq!(TextureResolution::R8192.label(), "8192 × 8192");
    }

    #[test]
    fn empty_name_becomes_untitled() {
        let project = ProjectStub::new("   ", TextureResolution::R2048);
        assert_eq!(project.name, DEFAULT_PROJECT_NAME);
        assert_eq!(project.texture_size(), (2048, 2048));
    }

    #[test]
    fn name_is_trimmed() {
        let project = ProjectStub::new("  ACE Logistics ", TextureResolution::R4096);
        assert_eq!(project.name, "ACE Logistics");
    }
}
