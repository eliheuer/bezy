# Bezy Font Editor: Comprehensive Refactoring Plan

*Complete refactoring strategy to make Bezy more accessible to LLMs, junior developers, and designers*

## 🔍 **Critical Issues Analysis**

### **1. Massive Dead Code Problem (36% bloat)**
- **Entire `/src_backup_tmp/` directory** (70 files!) - parallel codebase copy
- **250+ `#[allow(dead_code)]` annotations** across the codebase
- **Legacy compatibility layers** that aren't used
- **Unused import patterns** - 195 internal crate imports across 53 files

### **2. Monolithic Files Creating Complexity**
| File | Lines | Issues |
|------|-------|--------|
| `editing/selection/systems.rs` | 1,862 | Handles all selection logic in one place |
| `systems/text_editor_sorts.rs` | 1,442 | Hybrid text/visual editing system |
| `core/state/text_editor.rs` | 1,338 | Monolithic state structure |
| `ui/theme.rs` | 314+ | 100+ constants, most unused |

### **3. Plugin System Issues**
- **39 plugins with inconsistent patterns**
- **Circular dependencies** between UI, core, and rendering
- **No clear plugin lifecycle management**
- **Complex registration in `core/app.rs`**

### **4. State Management Problems**
```rust
// Current: Monolithic and confusing
pub struct AppState {
    pub workspace: Workspace,  // Everything mixed together
}

// Issues:
// - Large state objects passed around systems
// - No incremental dirty tracking
// - Limited state persistence
// - Unclear ownership patterns
```

## 📋 **Refactoring Plan**

### **Phase 1: Cleanup & Foundation (Week 1-2)**

#### **Priority 1: Remove Dead Weight**
```bash
# Immediate 36% size reduction
rm -rf src_backup_tmp/

# Find unused dependencies
cargo +nightly udeps

# Find unused code
cargo clippy -- -W unused
```

#### **Priority 2: Theme System Cleanup**
**Current Problem:**
```rust
// src/ui/theme.rs - 100+ constants, most unused
pub const UNUSED_CONSTANT_1: Color = Color::srgb(0.5, 0.5, 0.5);
pub const UNUSED_CONSTANT_2: f32 = 42.0;
// ... 50+ more unused constants
```

**Solution:**
```rust
// Organize by category, remove unused
pub mod colors {
    pub const BACKGROUND: Color = Color::hex("#1e1e1e").unwrap();
    pub const FOREGROUND: Color = Color::hex("#ffffff").unwrap();
    pub const ACCENT: Color = Color::hex("#ff6600").unwrap();
}

pub mod sizes {
    pub const UI_MARGIN: f32 = 8.0;
    pub const TOOLBAR_HEIGHT: f32 = 40.0;
    pub const BUTTON_SIZE: f32 = 32.0;
}
```

#### **Priority 3: Remove Legacy Compatibility**
```rust
// Remove unused workspace compatibility layer
// src/data/workspace.rs - entire file marked as unused
```

### **Phase 2: State Management Refactor (Week 3-5)**

#### **Current Problem: Monolithic State**
```rust
pub struct AppState {
    pub workspace: Workspace,  // Contains everything
}

pub struct Workspace {
    pub font: FontData,        // Large nested structure  
    pub info: FontInfo,        // Another large structure
    pub selected: Option<String>,  // Mixed concerns
}
```

#### **Proposed Solution: Modular State**
```rust
// Clear separation of concerns
pub struct AppState {
    pub font: FontState,         // Font data only
    pub editor: EditorState,     // UI and editing state  
    pub view: ViewportState,     // Camera and rendering
    pub selection: SelectionState, // Selection management
}

// Each state module has clear responsibility
pub trait StateModule {
    type Event;
    fn update(&mut self, event: Self::Event);
    fn get_dirty_regions(&self) -> &[DirtyRegion];
    fn clear_dirty(&mut self);
}

// Example implementation
impl StateModule for SelectionState {
    type Event = SelectionEvent;
    
    fn update(&mut self, event: SelectionEvent) {
        match event {
            SelectionEvent::SelectPoint(point_id) => {
                self.selected_points.insert(point_id);
                self.dirty_regions.push(DirtyRegion::Selection);
            }
            // ... other events
        }
    }
}
```

#### **Benefits:**
- **Better performance** through targeted updates
- **Easier testing** of individual components
- **Clear ownership** of state modifications
- **Optimized serialization/persistence**

### **Phase 3: Plugin Architecture Simplification (Week 6-7)**

#### **Current Problem: 39 Plugins with Complex Dependencies**
```rust
fn add_all_plugins(app: &mut App) {
    add_rendering_plugins(app);    // Order matters
    add_editor_plugins(app);       // but isn't explicit
    add_core_plugins(app);         // in dependencies
    app.add_plugins(GlyphGridPlugin); // Special case - why?
}
```

#### **Solution: Plugin Categories**
```rust
// Group related functionality
pub enum PluginCategory {
    Core,      // Input, state management
    Editing,   // Tools, selection, undo
    Rendering, // Drawing, visual feedback
    UI,        // Interface, themes
}

// Consistent plugin interface
pub trait BezyPlugin {
    fn id(&self) -> PluginId;
    fn category(&self) -> PluginCategory;
    fn dependencies(&self) -> &[PluginId];
    fn initialize(&mut self, app: &mut App);
    fn cleanup(&mut self, app: &mut App) {}
}

// Automatic dependency resolution
pub struct PluginManager {
    plugins: HashMap<PluginId, Box<dyn BezyPlugin>>,
    dependency_graph: DependencyGraph,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn BezyPlugin>) {
        let id = plugin.id();
        let deps = plugin.dependencies().to_vec();
        self.dependency_graph.add_node(id, deps);
        self.plugins.insert(id, plugin);
    }
    
    pub fn initialize_all(&mut self, app: &mut App) {
        let sorted = self.dependency_graph.topological_sort();
        for plugin_id in sorted {
            if let Some(plugin) = self.plugins.get_mut(&plugin_id) {
                plugin.initialize(app);
            }
        }
    }
}
```

### **Phase 4: Module Restructuring (Week 8-10)**

#### **Break Up Monolithic Files**

**Current: 1,862 line selection system**
```
src/editing/selection/systems.rs  # Everything in one file
```

**Proposed: Logical separation**
```
src/editing/selection/
├── mouse_selection.rs      # Mouse click/drag handling
├── keyboard_selection.rs   # Keyboard shortcuts
├── box_selection.rs       # Drag selection logic
├── multi_selection.rs     # Shift+click logic
├── selection_state.rs     # State management
└── mod.rs                 # Public API
```

#### **Standardize Naming Conventions**
```rust
// Current confusion: Sort vs SortEntry vs SortKind vs ActiveSort
// Proposed clarity:
pub struct Glyph { ... }           // The actual glyph data
pub struct GlyphEditor { ... }     // Editing operations on glyph
pub struct GlyphView { ... }       // Visual representation
pub struct GlyphSelection { ... }  // Selection state for glyph
```

#### **Clear Module Boundaries**
```rust
// Before: Confusing deep imports
use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use crate::editing::selection::components::{GlyphPointReference, PointType};

// After: Clear, shallow imports  
use crate::tools::SelectTool;
use crate::state::SelectionState;
use crate::events::SelectionEvent;
```

## 🎯 **Immediate Actions (This Week)**

### **Day 1-2: Dead Code Removal**
```bash
# Remove the obvious bloat
rm -rf src_backup_tmp/

# Find unused dependencies
cargo +nightly udeps

# Remove unused imports
cargo clippy -- -W unused

# Remove dead code allowances
grep -r "#\[allow(dead_code)\]" src/ | wc -l  # Count current
# Systematically remove unused items
```

### **Day 3-5: Theme System Cleanup**
1. **Audit `src/ui/theme.rs`** - identify actually used constants
2. **Organize by category** - colors, sizes, layouts, etc.
3. **Remove unused constants** - eliminate the 50+ unused items
4. **Update usage sites** - ensure consistent naming

```rust
// Example organized theme structure
pub mod theme {
    pub mod colors {
        pub const BACKGROUND: Color = Color::hex("#1e1e1e").unwrap();
        pub const FOREGROUND: Color = Color::hex("#ffffff").unwrap();
        pub const ACCENT: Color = Color::hex("#ff6600").unwrap();
        pub const SELECTION: Color = Color::hex("#ffff00").unwrap();
    }

    pub mod sizes {
        pub const TOOLBAR_HEIGHT: f32 = 40.0;
        pub const BUTTON_SIZE: f32 = 32.0;
        pub const MARGIN: f32 = 8.0;
        pub const PADDING: f32 = 4.0;
    }

    pub mod rendering {
        pub const POINT_RADIUS: f32 = 4.0;
        pub const LINE_WIDTH: f32 = 2.0;
        pub const ZOOM_STEP: f32 = 0.1;
    }
}
```

## 🧠 **LLM & Junior Developer Improvements**

### **1. Clear Module Boundaries**
- **Shallow import hierarchies** - max 3 levels deep
- **Consistent naming patterns** - same concepts use same words
- **Single responsibility** - each module does one thing well

### **2. Consistent Patterns**
```rust
// Every tool follows the same pattern
pub trait Tool {
    fn id(&self) -> ToolId;
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    fn shortcut(&self) -> Option<KeyCode>;
    
    // Core functionality
    fn handle_input(&mut self, input: &InputEvent);
    fn render(&self, gizmos: &mut Gizmos);
    fn get_cursor(&self) -> CursorStyle;
    
    // Lifecycle
    fn activate(&mut self) {}
    fn deactivate(&mut self) {}
}

// Example implementation
pub struct SelectTool {
    selected_points: HashSet<PointId>,
    drag_state: Option<DragState>,
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId { ToolId::Select }
    fn name(&self) -> &str { "Select" }
    fn icon(&self) -> &str { "\u{E001}" }
    fn shortcut(&self) -> Option<KeyCode> { Some(KeyCode::KeyV) }
    
    fn handle_input(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Click(position) => self.handle_click(*position),
            InputEvent::Drag(start, current) => self.handle_drag(*start, *current),
            // ... other events
        }
    }
}
```

### **3. Documentation Standards**
```rust
/// # SelectTool
/// 
/// Handles point selection with mouse and keyboard interactions.
/// 
/// ## Key Features
/// - **Single click**: Select individual point
/// - **Shift+click**: Add/remove from multi-selection  
/// - **Click+drag**: Box selection of multiple points
/// - **Arrow keys**: Nudge selected points
/// 
/// ## State Management
/// Updates `SelectionState` resource via `SelectionEvent` system.
/// 
/// ## Usage
/// ```rust
/// let mut tool = SelectTool::new();
/// tool.handle_input(&InputEvent::Click(position));
/// ```
pub struct SelectTool {
    /// Currently selected points
    selected_points: HashSet<PointId>,
    /// Drag selection state (if active)
    drag_state: Option<DragState>,
}
```

### **4. Example-Driven Development**
```rust
// Every complex concept includes working examples
pub mod examples {
    use super::*;
    
    /// Example: Creating a custom tool
    /// 
    /// This shows how to implement a simple custom tool
    /// that draws circles at clicked points.
    pub fn example_custom_tool() {
        struct CircleTool {
            circles: Vec<DVec2>,
        }
        
        impl Tool for CircleTool {
            fn handle_input(&mut self, input: &InputEvent) {
                if let InputEvent::Click(pos) = input {
                    self.circles.push(*pos);
                }
            }
            
            fn render(&self, gizmos: &mut Gizmos) {
                for circle_pos in &self.circles {
                    gizmos.circle_2d(*circle_pos, 10.0, Color::RED);
                }
            }
        }
    }
}
```

## 📊 **Expected Benefits**

### **Code Quality Improvements**
- **50% reduction in codebase size** (remove dead code)
- **Clear module boundaries** with single responsibilities
- **Consistent naming conventions** across all domains
- **Documented patterns** for common operations

### **Developer Experience Improvements**
- **Junior developers** can understand individual modules
- **LLMs get better context** from cleaner structure
- **Designers can modify themes** without touching logic
- **Plugin developers** have clear templates and examples

### **Performance Improvements**
- **Faster compilation** (less code to compile)
- **Better IDE performance** (clearer dependencies)
- **More efficient change detection** (modular state)
- **Reduced memory usage** (no dead code loaded)

### **Maintainability Improvements**
- **Easier debugging** (clear data flow)
- **Simpler testing** (isolated modules)
- **Better error messages** (clear ownership)
- **Reduced cognitive load** (consistent patterns)

## 🚀 **Getting Started Today**

### **Option 1: Big Bang Cleanup (Recommended)**
```bash
# Remove dead weight immediately
rm -rf src_backup_tmp/
git add -A && git commit -m "Remove backup directory (36% size reduction)"

# Audit and clean theme.rs
# This file has the most obvious improvements
```

### **Option 2: Incremental Approach**
1. **Pick one monolithic file** (like `selection/systems.rs`)
2. **Break it into logical pieces** following the proposed structure
3. **Update imports** to use the new structure
4. **Test that everything still works**
5. **Repeat with next file**

### **Option 3: Domain-Focused**
1. **Choose one domain** (like theme system)
2. **Clean it up completely** following new patterns
3. **Document the new patterns** with examples
4. **Use it as template** for other domains

## 🏗️ **Implementation Guidelines**

### **File Organization Rules**
1. **Max 200 lines per file** - if larger, split into logical pieces
2. **Max 3 levels of nesting** - keep imports shallow
3. **One main concept per file** - clear single responsibility
4. **Consistent naming** - same concepts use same words

### **Code Style Rules**
1. **Document all public APIs** - especially for plugin interfaces
2. **Include working examples** - in doc comments or examples/
3. **Use consistent error handling** - Result types with context
4. **Prefer composition** over inheritance (where applicable in Rust)

### **Testing Strategy**
1. **Unit tests for algorithms** - geometry, calculations
2. **Integration tests for plugins** - tool behavior, state changes
3. **Property tests for state** - invariants, consistency
4. **UI tests for interactions** - input/output behavior

## 🎮 **Performance Improvements**

### **1. Immediate Performance Wins**

#### **Clone Reduction (High Impact)**
- Replace 50+ `.clone()` calls with references where possible
- Use `Cow<str>` for strings that might be borrowed or owned
- Implement `Copy` trait for small structs like `PointData`

```rust
// Before
let glyph_name = sort.glyph_name.clone();

// After  
let glyph_name = &sort.glyph_name;
```

#### **System Scheduling Optimization**
- Add proper run conditions to systems
- Group related systems in sets with dependencies
- Use `Changed<T>` queries to avoid unnecessary work

```rust
// Example improved system scheduling
app.configure_sets(Update, (
    SelectionSystemSet::Input,
    SelectionSystemSet::Processing.after(SelectionSystemSet::Input),
    SelectionSystemSet::Render.after(SelectionSystemSet::Processing),
));
```

#### **Query Optimization**
- Split large queries into smaller, focused ones
- Use `Entity` instead of full component queries where possible
- Cache frequently accessed data in Local resources

### **2. Memory Management**
- Use object pools for frequently created/destroyed entities
- Implement smart batching for rendering operations
- Add memory usage monitoring in debug builds

## 📝 **Migration Checklist**

### **Phase 1 Complete When:**
- [ ] `src_backup_tmp/` removed
- [ ] Dead code allowances audited and cleaned
- [ ] Theme constants organized and documented
- [ ] Legacy compatibility layers removed
- [ ] All files compile without warnings

### **Phase 2 Complete When:**
- [ ] State split into logical modules
- [ ] Clear state ownership established
- [ ] Dirty tracking implemented
- [ ] State serialization working

### **Phase 3 Complete When:**
- [ ] Plugin categories defined
- [ ] Dependency resolution working
- [ ] Plugin lifecycle managed
- [ ] All plugins follow consistent pattern

### **Phase 4 Complete When:**
- [ ] No files over 200 lines
- [ ] Import depth max 3 levels
- [ ] Consistent naming throughout
- [ ] All public APIs documented

## 📈 **Metrics to Track**

- Frame time consistency
- Memory usage patterns
- System execution time
- Clone operation frequency
- Debug build performance vs release
- Number of dead code allowances
- Import depth across modules
- Documentation coverage

This comprehensive refactoring will transform Bezy from a complex, hard-to-navigate codebase into a clean, modular system that's much easier for both humans and LLMs to understand and extend. 