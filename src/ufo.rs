use anyhow::Result;
use norad::Font as Ufo;
use std::env;
use std::path::PathBuf;

pub fn load_ufo() {
    match load_ufo_from_args() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            println!(
                "Successfully loaded UFO font: {} {}",
                family_name, style_name
            );
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

pub fn load_ufo_from_args() -> Result<Ufo, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err("Usage: program <path-to-ufo-file>".into());
    }

    let font_path = PathBuf::from(&args[1]);
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}

pub fn get_basic_font_info() -> String {
    match load_ufo_from_args() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            format!("UFO: {} {}", family_name, style_name)
        }
        Err(e) => format!("UFO: Error loading font: {:?}", e),
    }
}
