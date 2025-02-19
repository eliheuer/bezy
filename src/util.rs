//! Shared helpers.

use std::convert::TryFrom;
use bevy::math::Vec2;
use bevy::prelude::Vec2 as BevyVec2;

/// Unwrap an optional, printing a message and returning if it is missing.
///
/// This should generate less code than unwrap? Honestly it's a total
/// experiment.
#[macro_export]
macro_rules! bail {
    ($opt:expr $(,)?) => {
        match $opt {
            Some(val) => val,
            None => {
                eprintln!("[{}:{}] bailed", file!(), line!());
                return
            }
        }
    };
     ($opt:expr, $($arg:tt)+) => {
        match $opt {
            Some(val) => val,
            None => {
                eprintln!("[{}:{}] bailed: ", file!(), line!());
                eprintln!($($arg)+);
                return
            }
        }
    };
}

/// Compute scale between two sizes, returning a Vec2
pub fn compute_scale(pre: (f32, f32), post: (f32, f32)) -> BevyVec2 {
    let ensure_finite = |f: f32| if f.is_finite() { f } else { 1.0 };
    let x = ensure_finite(post.0 / pre.0);
    let y = ensure_finite(post.1 / pre.1);
    BevyVec2::new(x, y)
}

/// Creates a new blank font with some placeholder glyphs.
pub fn create_blank_font() -> norad::Ufo {
    let mut ufo = norad::Ufo::new();
    ufo.font_info = norad::FontInfo {
        family_name: Some("Untitled".into()),
        style_name: Some("Regular".into()),
        units_per_em: Some(TryFrom::try_from(1000.0f64).unwrap()),
        descender: Some(From::from(-200.0)),
        ascender: Some(800.0.into()),
        cap_height: Some(700.0.into()),
        x_height: Some(500.0.into()),
        ..Default::default()
    }
    .into();

    let layer = ufo.get_default_layer_mut().unwrap();
    ('a'..='z')
        .into_iter()
        .chain('A'..='Z')
        .map(|chr| {
            let mut glyph = norad::Glyph::new_named(chr.to_string());
            glyph.codepoints = Some(vec![chr]);
            glyph
        })
        .for_each(|glyph| layer.insert_glyph(glyph));
    ufo
} 