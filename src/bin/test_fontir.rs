//! Test FontIR loading with designspace

use bezy::core::state::FontIRAppState;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let designspace_path = PathBuf::from("assets/fonts/BezyGrotesk.designspace");
    
    println!("Testing FontIR AppState with: {:?}", designspace_path);
    
    match FontIRAppState::from_path(designspace_path) {
        Ok(app_state) => {
            println!("Successfully created FontIR AppState!");
            println!("Source path: {:?}", app_state.source_path);
            println!("Current glyph: {:?}", app_state.current_glyph);
            println!("Glyph cache size: {}", app_state.glyph_cache.len());
            
            // Test getting current glyph paths
            println!("Current glyph paths: {:?}", app_state.get_current_glyph_paths().is_some());
        }
        Err(e) => {
            eprintln!("Failed to create FontIR AppState: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}