#[cfg(test)]
mod ufo_tests {
    use crate::ufo;
    

    #[test]
    fn test_load_ufo_from_path() {
        let test_path = "assets/fonts/bezy-grotesk-regular.ufo";
        let result = ufo::load_ufo_from_path(test_path);

        assert!(result.is_ok(), "Failed to load UFO file");

        let ufo = result.unwrap();

        // Test basic font info
        assert!(ufo.font_info.is_some(), "Font info should be present");

        if let Some(font_info) = ufo.font_info.as_ref() {
            assert_eq!(
                font_info.family_name.as_deref(),
                Some("Bezy Grotesk"),
                "Family name should match"
            );
            assert_eq!(
                font_info.style_name.as_deref(),
                Some("Regular"),
                "Style name should match"
            );
        }
    }
}

#[cfg(test)]
mod workspace_tests {
    use crate::{
        data::{AppState, Workspace},
        ufo,
    };
    use std::path::PathBuf;

    #[test]
    fn test_workspace_loads_ufo() {
        // First load the UFO file
        let test_path = "assets/fonts/bezy-grotesk-regular.ufo";
        let ufo = ufo::load_ufo_from_path(test_path)
            .expect("Failed to load UFO file");

        // Create a new workspace and set the font
        let mut workspace = Workspace::default();
        workspace.set_file(ufo, Some(PathBuf::from(test_path)));

        // Verify the workspace state
        assert_eq!(
            workspace.info.family_name, "Bezy Grotesk",
            "Workspace family name should match"
        );
        assert_eq!(
            workspace.info.style_name, "Regular",
            "Workspace style name should match"
        );

        // Test that we can create an AppState with this workspace
        let app_state = AppState { workspace };
        assert_eq!(
            app_state.get_font_display_name(),
            "Bezy Grotesk Regular",
            "App state should display correct font name"
        );
    }
}

// Add more test modules here as needed, for example:
// mod grid_tests { ... }
// mod font_info_tests { ... }
// etc.
