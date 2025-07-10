//! Text editor state and sort buffer management

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

/// Resource to track the active sort entity in ECS
/// This is the single source of truth for which sort is active
#[derive(Resource, Default)]
pub struct ActiveSortEntity {
    /// The entity ID of the currently active sort, if any
    pub entity: Option<Entity>,
    /// The buffer index of the active sort (for sync with buffer)
    pub buffer_index: Option<usize>,
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

/// An entry in the sort buffer representing a glyph or a line break
#[derive(Clone, Debug, PartialEq)]
pub enum SortKind {
    Glyph {
        glyph_name: String,
        advance_width: f32,
    },
    LineBreak,
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
    pub kind: SortKind,
    /// Whether this sort is currently active (in edit mode with points showing)
    pub is_active: bool,
    /// Whether this sort is currently selected (handle is highlighted)
    pub is_selected: bool,
    /// Layout mode for this sort
    pub layout_mode: SortLayoutMode,
    /// Root position (used for text buffer roots); for freeform sorts, use freeform_position
    pub root_position: Vec2,
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
            kind: SortKind::Glyph {
                glyph_name: String::new(),
                advance_width: 0.0,
            },
            is_active: false,
            is_selected: false,
            layout_mode: SortLayoutMode::Text,
            root_position: Vec2::ZERO,
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

        // Move the start of the gap to the deletion index.
        self.move_gap_to(index);

        // The element to be deleted is now at the end of the gap.
        // We "delete" it by incrementing the gap's end, effectively swallowing the element.
        let deleted_item = self.buffer[self.gap_end].clone();
        self.buffer[self.gap_end] = SortEntry::default(); // Clear the old slot
        self.gap_end += 1;
        
        Some(deleted_item)
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
                    kind: SortKind::Glyph {
                        glyph_name: glyph_name.to_string(),
                        advance_width: glyph_data.advance_width as f32,
                    },
                    is_active: false,
                    is_selected: false,
                    layout_mode: SortLayoutMode::Freeform, // Changed to Freeform
                    root_position: freeform_position, // Set actual position instead of Vec2::ZERO
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
            sort.root_position = freeform_position;
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
            sort.root_position = Vec2::ZERO;
            sort.buffer_index = Some(new_buffer_index);
            true
        } else {
            false
        }
    }
    
    /// Add a new freeform sort at the specified position
    pub fn add_freeform_sort(&mut self, glyph_name: String, position: Vec2, advance_width: f32) {
        // Clear all states first
        self.clear_all_states();
        
        let new_sort = SortEntry {
            kind: SortKind::Glyph {
                glyph_name: glyph_name.clone(),
                advance_width,
            },
            is_active: true, // Automatically activate the new sort
            is_selected: true, // Also select it
            layout_mode: SortLayoutMode::Freeform,
            root_position: position,
            buffer_index: None,
            is_buffer_root: false,
            buffer_cursor_position: None,
        };
        
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, new_sort);
        info!("Added and activated freeform sort '{}' at position ({:.1}, {:.1})", glyph_name, position.x, position.y);
    }
    
    /// Calculate flow position for text sorts
    pub fn get_text_sort_flow_position(&self, buffer_position: usize, font_metrics: &FontMetrics, leading: f32) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            if sort.layout_mode == SortLayoutMode::Text {
                let mut root_position = Vec2::ZERO;
                let mut x_offset = 0.0;
                let mut y_offset = 0.0;
                let mut found_root = false;
                let _line_height = (font_metrics.ascender.unwrap_or(1024.0) - font_metrics.descender.unwrap_or(-256.0)) as f32 + leading;
                
                debug!("Calculating flow position for buffer_position {} (glyph: '{}')", buffer_position, sort.kind.glyph_name());
                
                for i in (0..=buffer_position).rev() {
                    if let Some(candidate) = self.buffer.get(i) {
                        if candidate.is_buffer_root && candidate.layout_mode == SortLayoutMode::Text {
                            root_position = candidate.root_position;
                            found_root = true;
                            debug!("Found text root at index {} with position ({:.1}, {:.1})", i, root_position.x, root_position.y);
                            
                            // For the root itself, return the root position
                            if buffer_position == i {
                                debug!("This is the root, returning root position");
                                return Some(root_position);
                            }
                            
                            // For glyphs after the root, sum advance widths including the root's advance width
                            let mut total_advance = 0.0;
                            
                            // Add the root's advance width if it's a real glyph (not empty placeholder)
                            if candidate.kind.is_glyph() && !candidate.kind.glyph_name().is_empty() {
                                total_advance += candidate.kind.glyph_advance_width();
                                debug!("Added root advance width {} for glyph '{}', total: {:.1}", 
                                       candidate.kind.glyph_advance_width(), candidate.kind.glyph_name(), total_advance);
                            }
                            
                            // Add advance widths for all glyphs between root and current position
                            debug!("Summing advance widths from index {} to {}", i + 1, buffer_position);
                            for j in (i + 1)..buffer_position {
                                if let Some(text_sort) = self.buffer.get(j) {
                                    debug!("Processing index {}: glyph '{}'", j, text_sort.kind.glyph_name());
                                    match &text_sort.kind {
                                        SortKind::Glyph { advance_width, .. } => {
                                            total_advance += *advance_width;
                                            debug!("Added advance width {} for glyph '{}', total: {:.1}", 
                                                   advance_width, text_sort.kind.glyph_name(), total_advance);
                                        }
                                        SortKind::LineBreak => {
                                            total_advance = 0.0;
                                            // FIXED: Use proper line height calculation instead of descender - upm
                                            let upm = font_metrics.units_per_em as f32;
                                            let ascender = font_metrics.ascender.unwrap_or(1024.0) as f32;
                                            let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
                                            let line_height = (ascender - descender) + leading;
                                            y_offset -= line_height; // Move down by line height
                                            debug!("Line break: reset total_advance to 0.0, y_offset: {:.1} (line_height: {:.1})", y_offset, line_height);
                                        }
                                    }
                                }
                            }
                            
                            x_offset = total_advance;
                            debug!("Final total_advance: {:.1}, x_offset: {:.1}", total_advance, x_offset);
                            break;
                        }
                    }
                }
                if found_root {
                    let final_position = root_position + Vec2::new(x_offset, y_offset);
                    debug!("Calculated flow position for sort at index {}: root({:.1}, {:.1}) + offset({:.1}, {:.1}) = ({:.1}, {:.1})", 
                           buffer_position, root_position.x, root_position.y, x_offset, y_offset, final_position.x, final_position.y);
                    return Some(final_position);
                } else {
                    debug!("No root found for buffer_position {}", buffer_position);
                }
            } else {
                debug!("Sort at index {} is not in Text layout mode", buffer_position);
            }
        } else {
            debug!("No sort found at buffer_position {}", buffer_position);
        }
        None
    }

    /// Create a new text root at the specified world position
    pub fn create_text_root(&mut self, world_position: Vec2) {
        // Clear all states first
        self.clear_all_states();
        
        let text_root = SortEntry {
            kind: SortKind::Glyph {
                glyph_name: String::new(), // Empty glyph name for text root placeholder
                advance_width: 0.0,
            },
            is_active: true, // Automatically activate the new text root
            is_selected: true, // Select the new text root
            layout_mode: SortLayoutMode::Text,
            root_position: world_position,
            buffer_index: Some(self.buffer.len()),
            is_buffer_root: true,
            buffer_cursor_position: Some(0), // Start with cursor at position 0
        };
        
        // Insert at the end of the buffer
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);
        
        info!("Created and activated new text root at world position ({:.1}, {:.1}), cursor at position 0", 
              world_position.x, world_position.y);
    }
    
    /// Create a text sort at a specific world position (for text tool)
    pub fn create_text_sort_at_position(&mut self, glyph_name: String, world_position: Vec2, advance_width: f32) {
        // Only create a new root if there are no buffer roots yet
        if self.find_active_buffer_root_index().is_none() {
            // FIXED: Use the actual click position for the text root
            self.create_text_root(world_position);
        }
        // After root is created, insert the glyph at the cursor
        self.insert_sort_at_cursor(glyph_name, advance_width);
    }
    
    /// Get the visual position for a sort based on its layout mode
    pub fn get_sort_visual_position(&self, buffer_position: usize) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            match sort.layout_mode {
                SortLayoutMode::Text => {
                    // Text sorts now use their stored root_position
                    // But we need to calculate relative positions for text flow
                    if sort.is_buffer_root {
                        // Text roots use their exact stored position
                        Some(sort.root_position)
                    } else {
                        // Non-root text sorts flow from their text root
                        self.get_text_sort_flow_position(buffer_position, &FontMetrics::default(), 0.0)
                    }
                }
                SortLayoutMode::Freeform => {
                    Some(sort.root_position)
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
                        sort.kind.glyph_name(), i, sort_pos.x, sort_pos.y, handle_pos.x, handle_pos.y, world_position.x, world_position.y, distance, tolerance
                    );
                    
                    if distance < tolerance {
                        debug!("Found matching handle for sort '{}' at index {} (distance={:.1} < tolerance={:.1})", sort.kind.glyph_name(), i, distance, tolerance);
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
                        debug!("Found matching body for sort {} at index {}", sort.kind.glyph_name(), i);
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
            debug!("[activate_sort] Activated sort '{}' at buffer position {}", sort.kind.glyph_name(), position);
            debug!("[activate_sort] is_selected: {}, is_active: {}", sort.is_selected, sort.is_active);
            true
        } else {
            false
        }
    }
    
    /// Select a sort at the given buffer position (multiple can be selected)
    pub fn select_sort(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = true;
            debug!("Selected sort '{}' at buffer position {}", sort.kind.glyph_name(), position);
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
            debug!("Deselected sort '{}' at buffer position {}", sort.kind.glyph_name(), position);
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
            debug!("{} sort '{}' at buffer position {}", action, sort.kind.glyph_name(), position);
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
    
    /// Clear both active state and selections from all sorts
    pub fn clear_all_states(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
                sort.is_selected = false;
            }
        }
        debug!("Cleared all active states and selections from all sorts");
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
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos_in_buffer = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            info!("Inserting sort '{}' at cursor position {} in buffer root at index {}", 
                  glyph_name, cursor_pos_in_buffer, root_index);

            let new_sort = SortEntry {
                kind: SortKind::Glyph {
                    glyph_name: glyph_name.clone(),
                    advance_width,
                },
                is_active: false, // Don't make new sorts active by default
                is_selected: false, // Don't select new sorts by default
                layout_mode: SortLayoutMode::Text,
                root_position: Vec2::ZERO, // Will be calculated by flow
                buffer_index: None,
                is_buffer_root: false, // New sorts are not buffer roots
                buffer_cursor_position: None,
            };

            // If the buffer root is an empty placeholder, the new sort replaces it.
            if let Some(root_sort) = self.buffer.get(root_index) {
                if root_sort.is_buffer_root && root_sort.kind.is_glyph() && root_sort.kind.glyph_name().is_empty() {
                    // FIXED: Get the original root position before getting mutable reference
                    let original_root_position = root_sort.root_position;
                    if let Some(root_sort_mut) = self.buffer.get_mut(root_index) {
                        // FIXED: Preserve the root position from the original placeholder
                        *root_sort_mut = new_sort;
                        root_sort_mut.is_buffer_root = true;
                        root_sort_mut.is_active = true;
                        root_sort_mut.is_selected = true;
                        root_sort_mut.buffer_cursor_position = Some(1);
                        // FIXED: Set the root position to the original position
                        root_sort_mut.root_position = original_root_position;
                    }
                    info!("Replaced empty placeholder with new sort '{}' at position ({:.1}, {:.1})", 
                          glyph_name, original_root_position.x, original_root_position.y);
                    return;
                }
            }

            // For all other cases, insert at the correct position in the buffer sequence
            let insert_buffer_index = root_index + cursor_pos_in_buffer;
            self.buffer.insert(insert_buffer_index, new_sort);

            info!("Inserted sort '{}' at buffer index {} (root_index: {}, cursor_pos: {})", 
                  glyph_name, insert_buffer_index, root_index, cursor_pos_in_buffer);

            // Update the cursor position in the root.
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer + 1);
                info!("Updated cursor position to {} in buffer root", cursor_pos_in_buffer + 1);
            }
        } else {
            // No active text buffer, so create a new one with this character.
            info!("No active buffer root found, creating new text root with glyph '{}'", glyph_name);
            // FIXED: Use a reasonable default position instead of Vec2::ZERO
            let default_position = Vec2::new(500.0, 0.0);
            self.create_text_root_with_glyph(glyph_name, advance_width, default_position);
        }
    }
    
    /// Delete the sort at the cursor position
    pub fn delete_sort_at_cursor(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos_in_buffer = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            // Backspace does nothing if the cursor is at the beginning of the line.
            if cursor_pos_in_buffer == 0 {
                // We could implement joining with a previous line here in the future.
                return;
            }

            // A cursor at text position `k` is to the right of the glyph at `k-1`.
            // Backspace deletes the glyph to the left, which is at buffer index `root_index + k - 1`.
            let delete_buffer_index = root_index + cursor_pos_in_buffer - 1;

            self.buffer.delete(delete_buffer_index);

            // Update cursor position in the root
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer - 1);
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
                // A text sequence ends when we hit another buffer root or a non-text sort.
                if (i > root_index && sort.is_buffer_root) || sort.layout_mode != SortLayoutMode::Text {
                    break;
                }

                // The root placeholder doesn't count towards the string's length.
                if i == root_index && sort.kind.is_glyph() && sort.kind.glyph_name().is_empty() {
                    continue;
                }

                length += 1;
            } else {
                // End of the main buffer.
                break;
            }
        }
        length
    }

    /// Create a new line at the current cursor position
    /// This implements standard text editor behavior: insert a line break in the buffer
    pub fn create_new_line(&mut self, font_metrics: &FontMetrics) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos_in_buffer = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            let upm = font_metrics.units_per_em as f32;
            let descender = font_metrics.descender.unwrap_or(-256.0) as f32;

            let prev_root_y = self.buffer.get(root_index)
                .map(|root| root.root_position.y)
                .unwrap_or(0.0);
            let prev_root_x = self.buffer.get(root_index)
                .map(|root| root.root_position.x)
                .unwrap_or(0.0);

            let line_break = SortEntry {
                kind: SortKind::LineBreak,
                is_active: false,
                is_selected: false,
                layout_mode: SortLayoutMode::Text,
                root_position: Vec2::ZERO, // Not used for line breaks
                buffer_index: None,
                is_buffer_root: false,
                buffer_cursor_position: None,
            };
            let insert_index = root_index + cursor_pos_in_buffer;
            self.buffer.insert(insert_index, line_break);

            // Align new line's UPM with previous line's descender
            let new_root_y = prev_root_y + descender - upm;
            let new_root = SortEntry {
                kind: SortKind::Glyph {
                    glyph_name: String::new(),
                    advance_width: 0.0,
                },
                is_active: true,
                is_selected: true,
                layout_mode: SortLayoutMode::Text,
                root_position: Vec2::new(prev_root_x, new_root_y),
                buffer_index: Some(insert_index + 1),
                is_buffer_root: true,
                buffer_cursor_position: Some(0),
            };
            self.buffer.insert(insert_index + 1, new_root);

            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.is_selected = false;
                root_sort.is_active = false;
            }
        }
    }

    pub fn create_text_root_with_glyph(&mut self, glyph_name: String, advance_width: f32, world_position: Vec2) {
        // FIXED: Use the provided position instead of hardcoded position
        self.clear_all_states();

        let new_root = SortEntry {
            kind: SortKind::Glyph {
                glyph_name,
                advance_width,
            },
            is_active: true,
            is_selected: true,
            layout_mode: SortLayoutMode::Text,
            root_position: world_position,
            buffer_index: None,
            is_buffer_root: true,
            buffer_cursor_position: Some(1), // Cursor is after the typed character.
        };

        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, new_root);
    }

    /// Insert a line break at the cursor position (for Enter key)
    pub fn insert_line_break_at_cursor(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos_in_buffer = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            let new_sort = SortEntry {
                kind: SortKind::LineBreak,
                is_active: false,
                is_selected: false,
                layout_mode: SortLayoutMode::Text,
                root_position: Vec2::ZERO,
                buffer_index: None,
                is_buffer_root: false,
                buffer_cursor_position: None,
            };

            let insert_buffer_index = root_index + cursor_pos_in_buffer;
            self.buffer.insert(insert_buffer_index, new_sort);

            // Move cursor to the start of the new line (right after the line break)
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer + 1);
            }
        }
    }

    /// Move cursor up to the previous line (multi-line aware)
    pub fn move_cursor_up_multiline(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);
            // Build line starts and x offsets
            let mut line_starts = vec![0];
            let mut x_offsets = vec![0.0];
            let mut x = 0.0;
            for (i, entry) in self.buffer.iter().enumerate().skip(root_index + 1) {
                match &entry.kind {
                    SortKind::Glyph { advance_width, .. } => {
                        x += *advance_width;
                        x_offsets.push(x);
                    }
                    SortKind::LineBreak => {
                        line_starts.push(i - root_index);
                        x = 0.0;
                        x_offsets.push(x);
                    }
                }
            }
            // Find current line
            let mut current_line = 0;
            for (i, &start) in line_starts.iter().enumerate() {
                if cursor_pos < start {
                    break;
                }
                current_line = i;
            }
            if current_line == 0 {
                return; // Already at top line
            }
            let prev_line_start = line_starts[current_line - 1];
            let curr_x = x_offsets.get(cursor_pos).copied().unwrap_or(0.0);
            // Find closest x in previous line
            let prev_line_end = if current_line < line_starts.len() {
                line_starts[current_line]
            } else {
                x_offsets.len()
            };
            let mut best_idx = prev_line_start;
            let mut best_dist = f32::MAX;
            for idx in prev_line_start..prev_line_end {
                let dist = (x_offsets[idx] - curr_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = idx;
                }
            }
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(best_idx);
            }
        }
    }
    /// Move cursor down to the next line (multi-line aware)
    pub fn move_cursor_down_multiline(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos = self.buffer.get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);
            // Build line starts and x offsets
            let mut line_starts = vec![0];
            let mut x_offsets = vec![0.0];
            let mut x = 0.0;
            for (i, entry) in self.buffer.iter().enumerate().skip(root_index + 1) {
                match &entry.kind {
                    SortKind::Glyph { advance_width, .. } => {
                        x += *advance_width;
                        x_offsets.push(x);
                    }
                    SortKind::LineBreak => {
                        line_starts.push(i - root_index);
                        x = 0.0;
                        x_offsets.push(x);
                    }
                }
            }
            // Find current line
            let mut current_line = 0;
            for (i, &start) in line_starts.iter().enumerate() {
                if cursor_pos < start {
                    break;
                }
                current_line = i;
            }
            if current_line + 1 >= line_starts.len() {
                return; // Already at last line
            }
            let next_line_start = line_starts[current_line + 1];
            let next_line_end = if current_line + 2 < line_starts.len() {
                line_starts[current_line + 2]
            } else {
                x_offsets.len()
            };
            let curr_x = x_offsets.get(cursor_pos).copied().unwrap_or(0.0);
            // Find closest x in next line
            let mut best_idx = next_line_start;
            let mut best_dist = f32::MAX;
            for idx in next_line_start..next_line_end {
                let dist = (x_offsets[idx] - curr_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = idx;
                }
            }
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(best_idx);
            }
        }
    }
}

impl SortKind {
    pub fn is_glyph(&self) -> bool {
        matches!(self, SortKind::Glyph { .. })
    }
    pub fn is_line_break(&self) -> bool {
        matches!(self, SortKind::LineBreak)
    }
    pub fn glyph_name(&self) -> &str {
        match self {
            SortKind::Glyph { glyph_name, .. } => glyph_name,
            _ => "",
        }
    }
    pub fn glyph_advance_width(&self) -> f32 {
        match self {
            SortKind::Glyph { advance_width, .. } => *advance_width,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_activation_on_creation() {
        let mut text_editor = TextEditorState::default();
        
        // Test 1: Freeform sort should be activated when created
        text_editor.add_freeform_sort("a".to_string(), Vec2::new(100.0, 200.0), 500.0);
        
        // Verify the sort was created and activated
        assert_eq!(text_editor.buffer.len(), 1);
        if let Some(sort) = text_editor.buffer.get(0) {
            assert!(sort.is_active);
            assert!(sort.is_selected);
            assert_eq!(sort.kind.glyph_name(), "a");
            assert_eq!(sort.root_position, Vec2::new(100.0, 200.0));
        } else {
            panic!("Sort should exist at index 0");
        }
        
        // Test 2: Text sort should be activated when created
        text_editor.create_text_sort_at_position("b".to_string(), Vec2::new(300.0, 400.0), 600.0);
        
        // Verify the new sort was created and activated, and the old one was deactivated
        assert_eq!(text_editor.buffer.len(), 2);
        
        // First sort should be deactivated
        if let Some(sort) = text_editor.buffer.get(0) {
            assert!(!sort.is_active);
            assert!(!sort.is_selected);
        }
        
        // Second sort should be activated
        if let Some(sort) = text_editor.buffer.get(1) {
            assert!(sort.is_active);
            assert!(sort.is_selected);
            assert_eq!(sort.kind.glyph_name(), "b");
            assert_eq!(sort.root_position, Vec2::new(300.0, 400.0));
        } else {
            panic!("Sort should exist at index 1");
        }
        
        // Test 3: Text root should be activated when created
        text_editor.create_text_root(Vec2::new(500.0, 600.0));
        
        // Verify the new text root was created and activated, and others were deactivated
        assert_eq!(text_editor.buffer.len(), 3);
        
        // First two sorts should be deactivated
        for i in 0..2 {
            if let Some(sort) = text_editor.buffer.get(i) {
                assert!(!sort.is_active);
            }
        }
        
        // Third sort (text root) should be activated
        if let Some(sort) = text_editor.buffer.get(2) {
            assert!(sort.is_active);
            assert!(sort.is_selected);
            assert!(sort.is_buffer_root);
            assert_eq!(sort.root_position, Vec2::new(500.0, 600.0));
        } else {
            panic!("Text root should exist at index 2");
        }
    }

    #[test]
    fn test_text_flow_calculation() {
        let mut text_editor = TextEditorState::default();
        
        // Create a text root at position (100, 200)
        text_editor.create_text_root(Vec2::new(100.0, 200.0));
        println!("After create_text_root: buffer length = {}", text_editor.buffer.len());
        
        // Insert some glyphs with known advance widths
        text_editor.insert_sort_at_cursor("a".to_string(), 100.0);
        println!("After inserting 'a': buffer length = {}", text_editor.buffer.len());
        text_editor.insert_sort_at_cursor("b".to_string(), 150.0);
        println!("After inserting 'b': buffer length = {}", text_editor.buffer.len());
        text_editor.insert_sort_at_cursor("c".to_string(), 120.0);
        println!("After inserting 'c': buffer length = {}", text_editor.buffer.len());
        
        // Print buffer contents
        println!("\nBuffer contents:");
        for (i, sort) in text_editor.buffer.iter().enumerate() {
            println!("  [{}] '{}' (root: {}, active: {}, selected: {}) at ({:.1}, {:.1})", 
                     i, sort.kind.glyph_name(), sort.is_buffer_root, sort.is_active, sort.is_selected,
                     sort.root_position.x, sort.root_position.y);
        }
        
        // Verify the text flow positions
        let font_metrics = FontMetrics::default();
        
        // Root (placeholder) is at index 0
        if let Some(pos) = text_editor.get_text_sort_flow_position(0, &font_metrics, 0.0) {
            println!("Index 0 (root): calculated position = ({:.1}, {:.1})", pos.x, pos.y);
            assert_eq!(pos, Vec2::new(100.0, 200.0));
        } else {
            panic!("Should have flow position for root");
        }
        // First glyph after root is at index 1
        if let Some(pos) = text_editor.get_text_sort_flow_position(1, &font_metrics, 0.0) {
            println!("Index 1 (first glyph): calculated position = ({:.1}, {:.1})", pos.x, pos.y);
            assert_eq!(pos, Vec2::new(200.0, 200.0)); // 100 + 100
        } else {
            panic!("Should have flow position for first glyph");
        }
        // Second glyph after root is at index 2
        if let Some(pos) = text_editor.get_text_sort_flow_position(2, &font_metrics, 0.0) {
            println!("Index 2 (second glyph): calculated position = ({:.1}, {:.1})", pos.x, pos.y);
            assert_eq!(pos, Vec2::new(350.0, 200.0)); // 100 + 100 + 150
        } else {
            panic!("Should have flow position for second glyph");
        }
        // Third glyph after root is at index 2
        if let Some(pos) = text_editor.get_text_sort_flow_position(2, &font_metrics, 0.0) {
            println!("Index 2 (third glyph): calculated position = ({:.1}, {:.1})", pos.x, pos.y);
            assert_eq!(pos, Vec2::new(350.0, 200.0)); // 100 + 100 + 150
        } else {
            panic!("Should have flow position for third glyph");
        }
    }
} 