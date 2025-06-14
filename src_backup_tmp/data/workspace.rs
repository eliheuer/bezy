//! Virtual workspace management

use bevy::prelude::*;
use norad::{Glyph, GlyphName, Ufo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::state::{FontMetrics, SimpleFontInfo};

#[derive(Resource, Default)]
pub struct Workspace {
    #[allow(dead_code)]
    pub ufo: Option<Ufo>,
    #[allow(dead_code)]
    pub path: Option<PathBuf>,
    #[allow(dead_code)]
    pub selected_glyph: Option<GlyphName>,
    #[allow(dead_code)]
    pub open_glyphs: HashMap<GlyphName, Entity>,
    #[allow(dead_code)]
    pub info: SimpleFontInfo,
}

impl Workspace {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Workspace {
            ufo: None,
            path: None,
            selected_glyph: None,
            open_glyphs: HashMap::new(),
            info: SimpleFontInfo::default(),
        }
    }

    #[allow(dead_code)]
    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.ufo = Some(ufo);
        self.path = path;
        self.selected_glyph = None;
        self.open_glyphs.clear();
        self.info = SimpleFontInfo::default();
    }

    #[allow(dead_code)]
    pub fn get_glyph(&self, name: &GlyphName) -> Option<Arc<Glyph>> {
        self.ufo.as_ref()?.get_glyph(name).cloned()
    }

    #[allow(dead_code)]
    pub fn get_metrics(&self) -> FontMetrics {
        self.info.metrics.clone()
    }

    #[allow(dead_code)]
    pub fn get_display_name(&self) -> String {
        self.info.get_display_name()
    }
}
