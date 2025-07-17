# FontIR Migration Plan

## Data Structure Mapping

### Current → FontIR Replacement

| Current Type | FontIR Replacement | Notes |
|--------------|-------------------|--------|
| `FontData` | `DesignSpaceIrSource` + `fontir::orchestration::Context` | Holds the entire font project |
| `GlyphData` | `fontir::ir::Glyph` | Variable glyph with sources |
| `OutlineData` | `Vec<kurbo::BezPath>` | FontIR uses BezPath directly |
| `ContourData` | `kurbo::BezPath` | Single contour |
| `PointData` | `kurbo::PathEl` variants | Points are path elements |
| `PointTypeData` | `kurbo::PathEl` enum | Move/Line/Curve/Quad |

### Key Differences

1. **Variable Font Native**: FontIR stores multiple instances per glyph
2. **Path Representation**: Uses kurbo::BezPath instead of explicit points
3. **No Direct Point Access**: Points are part of path elements
4. **Component Support**: Built-in component references

## Migration Steps

### Phase 1: Core State (HIGH PRIORITY)
- [ ] Replace `Workspace` with FontIR context
- [ ] Replace `AppState::workspace.font` with `DesignSpaceIrSource`
- [ ] Update `AppState::get_point_mut()` to work with BezPath

### Phase 2: Selection System
- [ ] Convert point selection to path element indices
- [ ] Update `SelectedPoint` to reference PathEl position
- [ ] Modify selection visualization for BezPath

### Phase 3: Editing Operations
- [ ] Rewrite point movement for PathEl manipulation
- [ ] Update undo/redo for FontIR operations
- [ ] Convert tool operations to BezPath modifications

### Phase 4: Rendering
- [ ] Already uses kurbo for drawing - minimal changes
- [ ] Update outline extraction from FontIR glyphs
- [ ] Handle variable instance rendering

### Phase 5: File I/O
- [ ] Remove norad-based loading
- [ ] Use DesignSpaceIrSource for all formats
- [ ] Update save operations through FontIR

## Affected Files (20 total)

### Core Systems (Must Update First)
1. `src/core/state/font_data.rs` - DELETE (replaced by FontIR types)
2. `src/core/state/app_state.rs` - Major rewrite
3. `src/data/conversions.rs` - DELETE (no conversion needed)

### Selection & Editing
4. `src/editing/selection/systems.rs`
5. `src/editing/selection/entity_management/spawning.rs`
6. `src/geometry/point.rs`

### Rendering
7. `src/rendering/glyph_outline.rs`
8. `src/rendering/sort_visuals.rs`

### Tools
9. `src/tools/pen.rs`
10. `src/tools/pen_full.rs`
11. `src/ui/toolbars/edit_mode_toolbar/*.rs`

### Other Systems
12. `src/systems/sort_manager.rs`
13. `src/systems/text_editor_sorts/point_entities.rs`
14. `src/core/state/text_editor.rs`

## Benefits After Migration

1. **Multi-format support**: UFO, Glyphs, TTX via FontIR
2. **Variable fonts**: Native support, no custom implementation
3. **Components**: Automatic component handling
4. **Production ready**: Google Fonts-grade infrastructure
5. **Less code**: Remove ~500+ lines of custom data structures

## Challenges

1. **Point Selection**: Need new approach for selecting path elements
2. **Direct Point Access**: Must work through BezPath API
3. **Undo System**: Needs FontIR-aware operations
4. **Breaking Changes**: All editing code needs updates