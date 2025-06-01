//! Font data management and loading
//!
//! This module handles all font-related data operations:
//! - UFO (Unified Font Object) file format support
//! - Virtual font management for in-memory operations
//! - Workspace management for font projects
//! - Unicode utilities and character range definitions

pub mod ufo;
pub mod unicode;
pub mod virtual_font;
pub mod workspace;
