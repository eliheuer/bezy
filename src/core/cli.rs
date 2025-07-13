//! Command line interface for the Bezy font editor
//!
//! Handles parsing command line arguments and provides
//! validation for user inputs. Many CLI options are documented with
//! examples to help users understand the expected format.

use crate::core::settings::DEFAULT_UFO_PATH;
use crate::ui::themes::ThemeVariant;
use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Bezy CLI arguments
///
/// Examples:
///   bezy                                # Load default font
///   bezy --load-ufo my-font.ufo         # Load specific font
///   bezy --load-ufo ~/Fonts/MyFont.ufo  # Load font with full path
///   bezy --theme lightmode              # Use light mode theme
///   bezy --theme strawberry             # Use strawberry theme
#[derive(Parser, Debug, Resource)]
#[clap(
    name = "bezy",
    version,
    about = "A font editor built with Rust and Bevy",
    long_about = "Bezy is a cross-platform font editor that supports UFO (Unified Font Object) files. It provides glyph editing capabilities with a modern, game-engine-powered interface."
)]
pub struct CliArgs {
    /// Path to a UFO file to load on startup
    ///
    /// The UFO file should be a valid UFO version 3 directory structure.
    /// If not specified, loads the default sample font.
    #[clap(
        long = "load-ufo",
        short = 'f',
        default_value = DEFAULT_UFO_PATH,
        help = "UFO file to load",
        long_help = "Path to a UFO (Unified Font Object) file to load on startup. The file should be a valid UFO directory structure."
    )]
    pub ufo_path: Option<PathBuf>,

    /// Theme to use for the interface
    ///
    /// Available themes: darkmode (default), lightmode, strawberry, campfire.
    /// Custom themes can be added by creating new theme files.
    #[clap(
        long = "theme",
        short = 't',
        help = "Theme to use",
        long_help = "Theme to use for the interface. Available themes: darkmode (default), lightmode, strawberry, campfire"
    )]
    pub theme: Option<String>,
}

impl CliArgs {
    /// Validate the CLI arguments after parsing
    ///
    /// This ensures that all paths exist and are valid before the application starts,
    /// providing clear error messages for common mistakes.
    pub fn validate(&self) -> Result<(), String> {
        // Skip validation for WASM builds since filesystem works differently
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = &self.ufo_path {
                if !path.exists() {
                    return Err(format!(
                        "UFO file does not exist: {}\nMake sure the path is correct and the file exists.",
                        path.display()
                    ));
                }

                if !path.is_dir() {
                    return Err(format!(
                        "UFO path is not a directory: {}\nUFO files should be directories, not single files.",
                        path.display()
                    ));
                }

                // Check for required UFO files
                let meta_info = path.join("metainfo.plist");
                if !meta_info.exists() {
                    return Err(format!(
                        "Not a valid UFO file: missing metainfo.plist in {}\nMake sure this is a valid UFO directory.",
                        path.display()
                    ));
                }
            } else {
                // If no UFO path provided, use default
                return Err("Please provide a UFO file path as an argument.\nExample: bezy assets/fonts/bezy-grotesk-regular.ufo".to_string());
            }

            // Validate theme if provided
            if let Some(theme_name) = &self.theme {
                if ThemeVariant::from_str(theme_name).is_none() {
                    let available_themes = ThemeVariant::all_names().join(", ");
                    return Err(format!(
                        "Unknown theme: '{}'\nAvailable themes: {}",
                        theme_name, available_themes
                    ));
                }
            }

            Ok(())
        }
    }

    /// Create default CLI args for web builds
    ///
    /// For WASM builds, we use a default font path since command line arguments
    /// are not available in the browser environment.
    #[cfg(target_arch = "wasm32")]
    pub fn default_for_web() -> Self {
        Self {
            ufo_path: Some(PathBuf::from(DEFAULT_UFO_PATH)),
            theme: None, // Use default theme for web builds
        }
    }

    /// Get the UFO path, guaranteed to be Some after validation
    #[allow(dead_code)]
    pub fn get_ufo_path(&self) -> &PathBuf {
        self.ufo_path
            .as_ref()
            .expect("UFO path should be validated")
    }

    /// Get the theme variant from CLI args or default
    pub fn get_theme_variant(&self) -> ThemeVariant {
        self.theme
            .as_ref()
            .and_then(|theme_name| ThemeVariant::from_str(theme_name))
            .unwrap_or_default()
    }
}
