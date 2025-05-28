use bevy::prelude::*;
use norad::{Glyph, GlyphName, Ufo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::data::{FontMetrics, SimpleFontInfo};

#[derive(Resource, Default)]
pub struct Workspace {
    pub ufo: Option<Ufo>,
    pub path: Option<PathBuf>,
    pub selected_glyph: Option<GlyphName>,
    pub open_glyphs: HashMap<GlyphName, Entity>,
    pub info: SimpleFontInfo,
}

impl Workspace {
    pub fn new() -> Self {
        Workspace {
            ufo: None,
            path: None,
            selected_glyph: None,
            open_glyphs: HashMap::new(),
            info: SimpleFontInfo::default(),
        }
    }

    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.ufo = Some(ufo);
        self.path = path;
        self.selected_glyph = None;
        self.open_glyphs.clear();
        self.info = SimpleFontInfo::default();
    }

    pub fn get_glyph(&self, name: &GlyphName) -> Option<Arc<Glyph>> {
        self.ufo.as_ref()?.get_glyph(name).cloned()
    }

    pub fn get_metrics(&self) -> FontMetrics {
        self.info.metrics.clone()
    }

    pub fn get_display_name(&self) -> String {
        self.info.get_display_name()
    }
} 