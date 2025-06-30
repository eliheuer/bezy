//! Font data management and loading
//!
//! This module handles all font-related data operations:
//! - UFO (Unified Font Object) file format support
//! - UFO format conversions and serialization
//! - Workspace management for font projects
//! - Unicode utilities and character range definitions

pub mod conversions;
pub mod ufo;
pub mod unicode;
pub mod workspace; 