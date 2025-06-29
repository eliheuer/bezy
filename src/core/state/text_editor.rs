//! Text editor state and sort buffer management
//!
//! This module contains all the text editing functionality for the font editor,
//! including the gap buffer-based sort system that allows dynamic text composition.

use bevy::prelude::*;
use crate::core::state::{FontData, FontMetrics};

/// Text editor state for dynamic sort management
#[derive(Resource, Clone, Default)]
pub struct TextEditorState {
    /// The text buffer containing sort content (like a rope or gap buffer)
    pub buffer: SortBuffer,
    /// Current cursor position in the buffer
    pub cursor_position: usize,
    /// Selection range (start, end) if any
    pub selection: Option<(usize, usize)>,
    /// Viewport offset for scrolling
    pub viewport_offset: Vec2,
    /// Grid layout configuration
    pub grid_config: GridConfig,
}

/// Layout mode for individual sorts
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SortLayoutMode {
    /// Sort flows like text characters in a line
    #[default]
    Text,
    /// Sort is positioned freely in the design space
    Freeform,
}

/// Text mode configuration
#[derive(Resource, Clone, Debug, Default)]
pub struct TextModeConfig {
    /// Whether new sorts should be placed in text or freeform mode
    pub default_placement_mode: SortLayoutMode,
    /// Whether to show the mode toggle UI
    #[allow(dead_code)]
    pub show_mode_toggle: bool,
}

/// Unified buffer of all sorts (both text and freeform) using gap buffer for efficient editing
#[derive(Clone)]
pub struct SortBuffer {
    /// The gap buffer storage
    buffer: Vec<SortEntry>,
    /// Gap start position
    gap_start: usize,
    /// Gap end position (exclusive)
    gap_end: usize,
}

/// An entry in the sort buffer representing a glyph
#[derive(Clone, Debug)]
pub struct SortEntry {
    /// The name of the glyph this sort represents
    pub glyph_name: String,
    /// The advance width of the glyph (for spacing)
    pub advance_width: f32,
    /// Whether this sort is currently active (in edit mode with points showing)
    pub is_active: bool,
    /// Whether this sort is currently selected (handle is highlighted)
    pub is_selected: bool,
    /// Layout mode for this sort
    pub layout_mode: SortLayoutMode,
    /// Freeform position (only used when layout_mode is Freeform)
    pub freeform_position: Vec2,
    /// Buffer index (only used when layout_mode is Buffer)
    #[allow(dead_code)]
    pub buffer_index: Option<usize>,
    /// Whether this sort is a buffer root (first sort in a text buffer)
    pub is_buffer_root: bool,
    /// Cursor position within this buffer sequence (only for buffer roots)
    pub buffer_cursor_position: Option<usize>,
}

/// Grid layout configuration
#[derive(Clone)]
pub struct GridConfig {
    /// Number of sorts per row
    pub sorts_per_row: usize,
    /// Horizontal spacing between sorts
    pub horizontal_spacing: f32,
    /// Vertical spacing between rows
    pub vertical_spacing: f32,
    /// Starting position for the grid
    pub grid_origin: Vec2,
}

/// Iterator for gap buffer
pub struct SortBufferIterator<'a> {
    buffer: &'a SortBuffer,
    index: usize,
}

impl Default for SortBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SortEntry {
    fn default() -> Self {
        Self {
            glyph_name: String::new(),
            advance_width: 0.0,
            is_active: false,
            is_selected: false,
            layout_mode: SortLayoutMode::Text,
            freeform_position: Vec2::ZERO,
            buffer_index: None,
            is_buffer_root: false,
            buffer_cursor_position: None,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            sorts_per_row: 16,
            horizontal_spacing: 64.0,
            vertical_spacing: 400.0,
            grid_origin: Vec2::ZERO,
        }
    }
}

impl SortBuffer {
    /// Create a new gap buffer with initial capacity
    pub fn new() -> Self {
        let initial_capacity = 1024; // Start with room for plenty of sorts
        let mut buffer = Vec::with_capacity(initial_capacity);
        // Fill with default entries to create the gap
        buffer.resize(initial_capacity, SortEntry::default());
        
        Self {
            buffer,
            gap_start: 0,
            gap_end: initial_capacity,
        }
    }
    
    /// Create gap buffer from existing sorts (for font loading)
    #[allow(dead_code)]
    pub fn from_sorts(sorts: Vec<SortEntry>) -> Self {
        let len = sorts.len();
        let capacity = (len * 2).max(1024); // Double capacity for future edits
        let mut buffer = Vec::with_capacity(capacity);
        
        // Add the existing sorts
        buffer.extend(sorts);
        // Fill the rest with default entries to create the gap
        buffer.resize(capacity, SortEntry::default());
        
        Self {
            buffer,
            gap_start: len,
            gap_end: capacity,
        }
    }
    
    /// Get the logical length (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }
    
    /// Check if buffer is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get sort at logical position
    pub fn get(&self, index: usize) -> Option<&SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        if index < self.gap_start {
            self.buffer.get(index)
        } else {
            self.buffer.get(index + (self.gap_end - self.gap_start))
        }
    }
    
    /// Get mutable sort at logical position
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        if index < self.gap_start {
            self.buffer.get_mut(index)
        } else {
            self.buffer.get_mut(index + (self.gap_end - self.gap_start))
        }
    }
    
    /// Move gap to position for efficient insertion/deletion
    fn move_gap_to(&mut self, position: usize) {
        if position == self.gap_start {
            return;
        }
        
        if position < self.gap_start {
            // Move gap left
            let move_count = self.gap_start - position;
            let gap_size = self.gap_end - self.gap_start;
            
            // Move elements from before gap to after gap
            for i in 0..move_count {
                let src_idx = position + i;
                let dst_idx = self.gap_end - move_count + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }
            
            self.gap_start = position;
            self.gap_end = position + gap_size;
        } else {
            // Move gap right
            let move_count = position - self.gap_start;
            let gap_size = self.gap_end - self.gap_start;
            
            // Move elements from after gap to before gap
            for i in 0..move_count {
                let src_idx = self.gap_end + i;
                let dst_idx = self.gap_start + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }
            
            self.gap_start = position;
            self.gap_end = position + gap_size;
        }
    }
    
    /// Insert sort at position
    pub fn insert(&mut self, index: usize, sort: SortEntry) {
        if index > self.len() {
            return;
        }
        
        // Ensure we have space in the gap
        if self.gap_start >= self.gap_end {
            self.grow_gap();
        }
        
        // Move gap to insertion point
        self.move_gap_to(index);
        
        // Insert at gap start
        self.buffer[self.gap_start] = sort;
        self.gap_start += 1;
    }
    
    /// Delete sort at position
    pub fn delete(&mut self, index: usize) -> Option<SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        // Move gap to deletion point
        self.move_gap_to(index);
        
        // The element to delete is now just before the gap
        if self.gap_start > 0 {
            self.gap_start -= 1;
            let deleted = self.buffer[self.gap_start].clone();
            self.buffer[self.gap_start] = SortEntry::default();
            Some(deleted)
        } else {
            None
        }
    }
    
    /// Grow the gap when it gets too small
    fn grow_gap(&mut self) {
        let old_capacity = self.buffer.len();
        let new_capacity = old_capacity * 2;
        let gap_size = self.gap_end - self.gap_start;
        let new_gap_size = gap_size + (new_capacity - old_capacity);
        
        // Extend buffer
        self.buffer.resize(new_capacity, SortEntry::default());
        
        // Move elements after gap to end of new buffer
        let elements_after_gap = old_capacity - self.gap_end;
        if elements_after_gap > 0 {
            for i in (0..elements_after_gap).rev() {
                let src_idx = self.gap_end + i;
                let dst_idx = new_capacity - elements_after_gap + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
                self.buffer[src_idx] = SortEntry::default();
            }
        }
        
        // Update gap end
        self.gap_end = self.gap_start + new_gap_size;
    }
    
    /// Get an iterator over all sorts (excluding gap)
    pub fn iter(&self) -> SortBufferIterator {
        SortBufferIterator {
            buffer: self,
            index: 0,
        }
    }
    
    /// Clear all sorts and reset gap
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = SortEntry::default();
        }
        self.gap_start = 0;
        self.gap_end = self.buffer.len();
    }
    
    /// Get all sorts as a vector (for debugging/serialization)
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<SortEntry> {
        let mut result = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            if let Some(sort) = self.get(i) {
                result.push(sort.clone());
            }
        }
        result
    }
}

impl<'a> Iterator for SortBufferIterator<'a> {
    type Item = &'a SortEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.buffer.len() {
            None
        } else {
            let item = self.buffer.get(self.index);
            self.index += 1;
            item
        }
    }
}

impl TextEditorState {
    /// Create a new text editor state from font data
    /// All initial sorts are created as freeform sorts arranged in a grid
    #[allow(dead_code)]
    pub fn from_font_data(font_data: &FontData) -> Self {
        // Convert font glyphs to sort entries in alphabetical order
        let mut glyph_names: Vec<_> = font_data.glyphs.keys().collect();
        glyph_names.sort();
        
        let mut sorts = Vec::new();
        let grid_config = GridConfig::default();
        
        for (index, glyph_name) in glyph_names.iter().enumerate() {
            if let Some(glyph_data) = font_data.glyphs.get(*glyph_name) {
                // Calculate freeform position in overview grid
                let row = index / grid_config.sorts_per_row;
                let col = index % grid_config.sorts_per_row;
                
                let freeform_position = Vec2::new(
                    grid_config.grid_origin.x + col as f32 * (1000.0 + grid_config.horizontal_spacing),
                    grid_config.grid_origin.y - row as f32 * (1200.0 + grid_config.vertical_spacing),
                );
                
                sorts.push(SortEntry {
                    glyph_name: glyph_name.to_string(),
                    advance_width: glyph_data.advance_width as f32,
                    is_active: false,
                    is_selected: false,
                    layout_mode: SortLayoutMode::Freeform, // Changed to Freeform
                    freeform_position, // Set actual position instead of Vec2::ZERO
                    buffer_index: None, // No buffer index for freeform sorts
                    is_buffer_root: false, // Overview sorts are not buffer roots
                    buffer_cursor_position: None, // No cursor for freeform sorts
                });
            }
        }
        
        let buffer = SortBuffer::from_sorts(sorts);
        
        Self {
            buffer,
            cursor_position: 0,
            selection: None,
            viewport_offset: Vec2::ZERO,
            grid_config,
        }
    }
    
    /// Get all sorts (both buffer and freeform)
    #[allow(dead_code)]
    pub fn get_all_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut all_sorts = Vec::new();
        
        // Add buffer sorts
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                all_sorts.push((i, sort));
            }
        }
        
        all_sorts
    }
    
    /// Get only text sorts (sorts that flow like text)
    pub fn get_text_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut text_sorts = Vec::new();
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Text {
                    text_sorts.push((i, sort));
                }
            }
        }
        
        text_sorts
    }
    
    /// Get only freeform sorts
    #[allow(dead_code)]
    pub fn get_freeform_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut freeform_sorts = Vec::new();
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Freeform {
                    freeform_sorts.push((i, sort));
                }
            }
        }
        
        freeform_sorts
    }
    
    /// Convert a sort from text mode to freeform mode
    #[allow(dead_code)]
    pub fn convert_sort_to_freeform(&mut self, buffer_position: usize, freeform_position: Vec2) -> bool {
        if let Some(sort) = self.buffer.get_mut(buffer_position) {
            sort.layout_mode = SortLayoutMode::Freeform;
            sort.freeform_position = freeform_position;
            sort.buffer_index = None;
            true
        } else {
            false
        }
    }
    
    /// Convert a sort from freeform mode to text mode
    #[allow(dead_code)]
    pub fn convert_sort_to_text(&mut self, buffer_position: usize, new_buffer_index: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(buffer_position) {
            sort.layout_mode = SortLayoutMode::Text;
            sort.freeform_position = Vec2::ZERO;
            sort.buffer_index = Some(new_buffer_index);
            true
        } else {
            false
        }
    }
    
    /// Add a new freeform sort at the specified position
    pub fn add_freeform_sort(&mut self, glyph_name: String, position: Vec2, advance_width: f32) {
        let new_sort = SortEntry {
            glyph_name: glyph_name.clone(),
            advance_width,
            is_active: false,
            is_selected: false,
            layout_mode: SortLayoutMode::Freeform,
            freeform_position: position,
            buffer_index: None,
            is_buffer_root: false,
            buffer_cursor_position: None,
        };
        
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, new_sort);
        info!("Added freeform sort '{}' at position ({:.1}, {:.1})", glyph_name, position.x, position.y);
    }
    
    /// Calculate flow position for text sorts
    fn get_text_sort_flow_position(&self, buffer_position: usize) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            if sort.layout_mode == SortLayoutMode::Text {
                // Find the text buffer root for this sort
                let mut root_position = Vec2::ZERO;
                let mut x_offset = 0.0;
                
                // Find the buffer root by searching backwards
                for i in (0..=buffer_position).rev() {
                    if let Some(candidate) = self.buffer.get(i) {
                        if candidate.is_buffer_root && candidate.layout_mode == SortLayoutMode::Text {
                            root_position = candidate.freeform_position;
                            
                            // Calculate cumulative advance width from root to current position
                            for j in i..buffer_position {
                                if let Some(text_sort) = self.buffer.get(j) {
                                    if text_sort.layout_mode == SortLayoutMode::Text && !text_sort.glyph_name.is_empty() {
                                        x_offset += text_sort.advance_width;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                
                return Some(root_position + Vec2::new(x_offset, 0.0));
            }
        }
        None
    }

    /// Create a new text root at the specified world position
    pub fn create_text_root(&mut self, world_position: Vec2) {
        let text_root = SortEntry {
            glyph_name: String::new(), // Empty glyph name for text root placeholder
            advance_width: 0.0,
            is_active: false,
            is_selected: true, // Select the new text root
            layout_mode: SortLayoutMode::Text,
            freeform_position: world_position,
            buffer_index: Some(self.buffer.len()),
            is_buffer_root: true,
            buffer_cursor_position: Some(0), // Start with cursor at position 0
        };
        
        // Insert at the end of the buffer
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);
        
        // FIXED: Position cursor at the text root so typing replaces it and moves cursor forward
        // This ensures when user types the first character, cursor ends up after that character
        self.cursor_position = insert_index;
        
        info!("Created new text root at world position ({:.1}, {:.1}), cursor at position {}", 
              world_position.x, world_position.y, self.cursor_position);
    }
    
    /// Create a text sort at a specific world position (for text tool)
    pub fn create_text_sort_at_position(&mut self, glyph_name: String, world_position: Vec2, advance_width: f32) {
        // Always create each clicked text sort as a new text root to preserve exact positioning
        // This ensures text sorts stay exactly where clicked, just like freeform sorts
        let text_root = SortEntry {
            glyph_name: glyph_name.clone(),
            advance_width,
            is_active: false,
            is_selected: true, // Select the new text root
            layout_mode: SortLayoutMode::Text,
            freeform_position: world_position,
            buffer_index: Some(self.buffer.len()),
            is_buffer_root: true, // Always make clicked text sorts into roots
            buffer_cursor_position: Some(1), // Position cursor after the new sort for typing
        };
        
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);
        
        // FIXED: Position cursor after the new sort for immediate typing continuation
        // Since we inserted a real glyph (not empty), cursor should be positioned to continue typing
        self.cursor_position = insert_index + 1;
        
        info!("Created new text root '{}' at world position ({:.1}, {:.1}), cursor at position {}", 
              glyph_name, world_position.x, world_position.y, self.cursor_position);
    }
    
    /// Get the visual position for a sort based on its layout mode
    pub fn get_sort_visual_position(&self, buffer_position: usize) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            match sort.layout_mode {
                SortLayoutMode::Text => {
                    // Text sorts now use their stored freeform_position
                    // But we need to calculate relative positions for text flow
                    if sort.is_buffer_root {
                        // Text roots use their exact stored position
                        Some(sort.freeform_position)
                    } else {
                        // Non-root text sorts flow from their text root
                        self.get_text_sort_flow_position(buffer_position)
                    }
                }
                SortLayoutMode::Freeform => {
                    Some(sort.freeform_position)
                }
            }
        } else {
            None
        }
    }

    /// Find a sort handle at a given world position (for freeform sorts)
    pub fn find_sort_handle_at_position(
        &self,
        world_position: Vec2,
        tolerance: f32,
        font_metrics: Option<&FontMetrics>,
    ) -> Option<usize> {
        // Check handles for all sorts (both buffer and freeform have handles)
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    let descender = if let Some(metrics) = font_metrics {
                        metrics.descender.unwrap_or(-200.0) as f32
                    } else {
                        -200.0 // Default descender value
                    };
                    // Match the rendering logic exactly: handle_position = world_pos + Vec2::new(0.0, descender)
                    let handle_pos = sort_pos + Vec2::new(0.0, descender);
                    let distance = world_position.distance(handle_pos);
                    
                    debug!(
                        "Checking sort '{}' at index {}: sort_pos=({:.1}, {:.1}), handle_pos=({:.1}, {:.1}), click_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                        sort.glyph_name, i, sort_pos.x, sort_pos.y, handle_pos.x, handle_pos.y, world_position.x, world_position.y, distance, tolerance
                    );
                    
                    if distance < tolerance {
                        debug!("Found matching handle for sort '{}' at index {} (distance={:.1} < tolerance={:.1})", sort.glyph_name, i, distance, tolerance);
                        return Some(i);
                    }
                }
            }
        }
        debug!("No handle found at position ({:.1}, {:.1}) with tolerance {:.1}", world_position.x, world_position.y, tolerance);
        None
    }

    /// Find a sort body at a given world position
    pub fn find_sort_body_at_position(&self, world_position: Vec2, tolerance: f32) -> Option<usize> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    if world_position.distance(sort_pos) < tolerance {
                        debug!("Found matching body for sort {} at index {}", sort.glyph_name, i);
                        return Some(i);
                    }
                }
            }
        }
        None
    }
    
    /// Get the sort at a specific buffer position
    pub fn get_sort_at_position(&self, position: usize) -> Option<&SortEntry> {
        self.buffer.get(position)
    }
    
    /// Get the currently active sort
    pub fn get_active_sort(&self) -> Option<(usize, &SortEntry)> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_active {
                    return Some((i, sort));
                }
            }
        }
        None
    }
    
    /// Activate a sort at the given buffer position (only one can be active)
    pub fn activate_sort(&mut self, position: usize) -> bool {
        // First deactivate all sorts
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        
        // Then activate the specified sort
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_active = true;
            debug!("Activated sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Select a sort at the given buffer position (multiple can be selected)
    pub fn select_sort(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = true;
            debug!("Selected sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Deselect a sort at the given buffer position
    #[allow(dead_code)]
    pub fn deselect_sort(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = false;
            debug!("Deselected sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Toggle selection state of a sort at the given buffer position
    pub fn toggle_sort_selection(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = !sort.is_selected;
            let action = if sort.is_selected { "Selected" } else { "Deselected" };
            debug!("{} sort '{}' at buffer position {}", action, sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Clear all selections
    pub fn clear_selections(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_selected = false;
            }
        }
        debug!("Cleared all sort selections");
    }
    
    /// Get all currently selected sorts
    pub fn get_selected_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut selected_sorts = Vec::new();
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_selected {
                    selected_sorts.push((i, sort));
                }
            }
        }
        selected_sorts
    }
    
    /// Check if a sort at the given position is selected
    #[allow(dead_code)]
    pub fn is_sort_selected(&self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get(position) {
            sort.is_selected
        } else {
            false
        }
    }
    
    /// Clear active state from all sorts
    pub fn clear_active_state(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        debug!("Cleared active state from all sorts");
    }
    
    /// Get the visual position (world coordinates) for a buffer position
    #[allow(dead_code)]
    pub fn get_world_position_for_buffer_position(&self, buffer_position: usize) -> Vec2 {
        let row = buffer_position / self.grid_config.sorts_per_row;
        let col = buffer_position % self.grid_config.sorts_per_row;
        
        let x = col as f32 * (1000.0 + self.grid_config.horizontal_spacing);
        let y = -(row as f32) * (1200.0 + self.grid_config.vertical_spacing);
        
        self.grid_config.grid_origin + Vec2::new(x, y)
    }
    
    /// Get the buffer position for a world coordinate (for click detection)
    pub fn get_buffer_position_for_world_position(&self, world_pos: Vec2) -> Option<usize> {
        let relative_pos = world_pos - self.grid_config.grid_origin;
        
        // Calculate grid row and column
        let col = (relative_pos.x / (1000.0 + self.grid_config.horizontal_spacing)).floor() as usize;
        
        // Handle negative Y coordinates correctly for downward-growing grid
        let row = if relative_pos.y <= 0.0 {
            ((-relative_pos.y) / (1200.0 + self.grid_config.vertical_spacing)).floor() as usize
        } else {
            0
        };
        
        // Convert grid position to buffer position
        let buffer_position = row * self.grid_config.sorts_per_row + col;
        
        // Validate the position is within bounds
        if buffer_position < self.buffer.len() {
            Some(buffer_position)
        } else {
            None
        }
    }
    
    /// Insert a new sort at the cursor position (for typing)
    pub fn insert_sort_at_cursor(&mut self, glyph_name: String, advance_width: f32) {
        // Find an active buffer root using more robust logic
        let mut buffer_root_index = None;
        let mut cursor_pos_in_buffer = 0;
        
        // FIXED: Use more robust logic to find active buffer root
        // First try to find a selected buffer root
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    buffer_root_index = Some(i);
                    cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                    break;
                }
            }
        }
        
        // If no selected buffer root found, look for any buffer root with a cursor position
        if buffer_root_index.is_none() {
            for i in 0..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                        break;
                    }
                }
            }
        }
        
        // If still no buffer root found, look for the most recently added buffer root
        if buffer_root_index.is_none() {
            for i in (0..self.buffer.len()).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = 0; // Start at beginning if no cursor position set
                        // Set the cursor position on this buffer root
                        break;
                    }
                }
            }
        }
        
        if let Some(root_index) = buffer_root_index {
            // We have an active buffer root - handle insertion within this buffer sequence
            let actual_insert_position = root_index + cursor_pos_in_buffer;
            
            // Check if we're replacing an empty buffer root
            if cursor_pos_in_buffer == 0 {
                if let Some(root_sort) = self.buffer.get(root_index) {
                    if root_sort.glyph_name.is_empty() && root_sort.is_buffer_root {
                        // Replace the empty buffer root with the typed character
                        if let Some(sort) = self.buffer.get_mut(root_index) {
                            sort.glyph_name = glyph_name.clone();
                            sort.advance_width = advance_width;
                            sort.is_active = true;
                            // Update cursor position within this buffer
                            sort.buffer_cursor_position = Some(1);
                        }
                        info!("Replaced empty buffer root with '{}', cursor now at position 1", glyph_name);
                        return;
                    }
                }
            }
            
            // Calculate position for the new sort based on text flow from buffer root
            let buffer_root_position = self.buffer.get(root_index)
                .map(|sort| sort.freeform_position)
                .unwrap_or(Vec2::ZERO);
            
            // Calculate cumulative advance width up to cursor position
            let mut x_offset = 0.0;
            for i in 0..cursor_pos_in_buffer {
                let sort_index = root_index + i;
                if let Some(sort) = self.buffer.get(sort_index) {
                    if sort.layout_mode == SortLayoutMode::Text && !sort.glyph_name.is_empty() {
                        x_offset += sort.advance_width;
                    }
                }
            }
            
            let new_position = buffer_root_position + Vec2::new(x_offset, 0.0);
            
            let new_sort = SortEntry {
                glyph_name: glyph_name.clone(),
                advance_width,
                is_active: false,
                is_selected: false,
                layout_mode: SortLayoutMode::Text,
                freeform_position: new_position,
                buffer_index: Some(actual_insert_position),
                is_buffer_root: false,
                buffer_cursor_position: None,
            };
            
            // Insert the new sort
            self.buffer.insert(actual_insert_position, new_sort);
            
            // Update the buffer root's cursor position
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer + 1);
            }
            
            info!("Inserted '{}' at buffer position {}, cursor now at position {}", 
                  glyph_name, actual_insert_position, cursor_pos_in_buffer + 1);
        } else {
            // No active buffer root found - this shouldn't happen in normal typing flow
            warn!("No active buffer root found for typing - this should not happen");
        }
    }
    
    /// Delete the sort at the cursor position
    pub fn delete_sort_at_cursor(&mut self) {
        // FIXED: Use per-buffer-root cursor system instead of global cursor position
        // Find the active buffer root and delete from its cursor position
        let mut buffer_root_index = None;
        let mut cursor_pos_in_buffer = 0;
        
        // Find active buffer root using the same logic as insert_sort_at_cursor
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    buffer_root_index = Some(i);
                    cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                    break;
                }
            }
        }
        
        if buffer_root_index.is_none() {
            for i in 0..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                        break;
                    }
                }
            }
        }
        
        if buffer_root_index.is_none() {
            for i in (0..self.buffer.len()).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = 0;
                        break;
                    }
                }
            }
        }
        
        if let Some(root_index) = buffer_root_index {
            if cursor_pos_in_buffer > 0 {
                let delete_position = root_index + cursor_pos_in_buffer - 1;
                if delete_position < self.buffer.len() {
                    self.buffer.delete(delete_position);
                    // Update the buffer root's cursor position
                    if let Some(root_sort) = self.buffer.get_mut(root_index) {
                        root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer - 1);
                    }
                    info!("Deleted sort at position {}, cursor now at position {}", 
                          delete_position, cursor_pos_in_buffer - 1);
                }
            } else {
                // At position 0 - check if this is an empty buffer root we can delete
                if let Some(root_sort) = self.buffer.get(root_index) {
                    if root_sort.glyph_name.is_empty() && root_sort.is_buffer_root {
                        self.buffer.delete(root_index);
                        info!("Deleted empty buffer root at position {}", root_index);
                    }
                }
            }
        }
    }
    
    /// Move cursor to a specific position (now works with per-buffer-root cursors)
    pub fn move_cursor_to(&mut self, position: usize) {
        // FIXED: Update the active buffer root's cursor position instead of global cursor
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                // Clamp the position to valid range for this buffer sequence
                let clamped_position = position.min(max_pos);
                root_sort.buffer_cursor_position = Some(clamped_position);
                info!("Moved cursor to position {} in buffer root at index {}", 
                      clamped_position, root_index);
            }
        }
    }
    
    /// Move cursor left by one position
    pub fn move_cursor_left(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos > 0 {
                    root_sort.buffer_cursor_position = Some(current_pos - 1);
                    info!("Moved cursor left to position {} in buffer root at index {}", 
                          current_pos - 1, root_index);
                }
            }
        }
    }
    
    /// Move cursor right by one position
    pub fn move_cursor_right(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos < max_pos {
                    root_sort.buffer_cursor_position = Some(current_pos + 1);
                    info!("Moved cursor right to position {} in buffer root at index {}", 
                          current_pos + 1, root_index);
                }
            }
        }
    }
    
    /// Move cursor up by one row (for buffer mode, move to previous buffer root)
    pub fn move_cursor_up(&mut self) {
        // FIXED: In buffer mode, move to previous buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the previous buffer root
            for i in (0..current_root_index).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        // Deselect current buffer root and select previous one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_selected = false;
                        }
                        // Get buffer_length before getting mutable reference to avoid borrow conflicts
                        let buffer_length = self.get_buffer_sequence_length(i);
                        if let Some(prev_root) = self.buffer.get_mut(i) {
                            prev_root.is_selected = true;
                            // Set cursor to end of previous buffer
                            prev_root.buffer_cursor_position = Some(buffer_length);
                            info!("Moved up to buffer root at index {}, cursor at position {}", 
                                  i, buffer_length);
                        }
                        return;
                    }
                }
            }
        }
    }
    
    /// Move cursor down by one row (for buffer mode, move to next buffer root)
    pub fn move_cursor_down(&mut self) {
        // FIXED: In buffer mode, move to next buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the next buffer root
            for i in (current_root_index + 1)..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        // Deselect current buffer root and select next one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_selected = false;
                        }
                        if let Some(next_root) = self.buffer.get_mut(i) {
                            next_root.is_selected = true;
                            // Set cursor to beginning of next buffer
                            next_root.buffer_cursor_position = Some(0);
                            info!("Moved down to buffer root at index {}, cursor at position 0", i);
                        }
                        return;
                    }
                }
            }
        }
    }
    
    /// Helper: Find the index of the active buffer root
    fn find_active_buffer_root_index(&self) -> Option<usize> {
        // Use same logic as insert_sort_at_cursor
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    return Some(i);
                }
            }
        }
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                    return Some(i);
                }
            }
        }
        
        for i in (0..self.buffer.len()).rev() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root {
                    return Some(i);
                }
            }
        }
        
        None
    }
    
    /// Helper: Get the length of a buffer sequence starting from a buffer root
    fn get_buffer_sequence_length(&self, root_index: usize) -> usize {
        let mut length = 0;
        for i in root_index..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Text {
                    if i == root_index || !sort.is_buffer_root {
                        length += 1;
                    } else {
                        // Hit another buffer root, stop counting
                        break;
                    }
                } else {
                    // Hit a freeform sort, stop counting
                    break;
                }
            }
        }
        length
    }
} 