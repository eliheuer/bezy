# Bezy Font Editor - Refactoring Plan

## Performance & Maintainability Improvements

### 1. **Immediate Performance Wins**

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

### 2. **Code Organization for Junior Developers**

#### **Consistent Error Handling**
- Create custom error types with clear messages
- Use `Result<T, BezyError>` consistently
- Add error context with `anyhow::Context`

#### **Documentation Standards**
- Add module-level documentation explaining purpose
- Document all public functions with examples
- Include performance notes for complex operations

#### **Type Safety Improvements**
- Use newtypes for indices: `ContourIndex(usize)`, `PointIndex(usize)`
- Replace magic numbers with named constants
- Add validation at API boundaries

### 3. **Architectural Improvements**

#### **Event-Driven Architecture**
```rust
// Replace direct state mutation with events
#[derive(Event)]
pub struct PointMoved {
    pub glyph_name: String,
    pub contour_idx: usize,
    pub point_idx: usize,
    pub new_position: Vec2,
}
```

#### **Plugin Organization**
- Split large plugins into smaller, focused ones
- Use consistent naming: `<Feature>Plugin`
- Group related functionality together

#### **Memory Management**
- Use object pools for frequently created/destroyed entities
- Implement smart batching for rendering operations
- Add memory usage monitoring in debug builds

### 4. **Testing & Debugging**

#### **Unit Testing Structure**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_point_movement_updates_glyph_data() {
        // Test implementation
    }
}
```

#### **Debug Tools**
- Add performance profiling markers
- Create debug overlays for system timing
- Implement memory usage visualization

### 5. **Implementation Priority**

1. **Week 1**: Clone reduction and basic performance wins
2. **Week 2**: System scheduling and query optimization  
3. **Week 3**: Error handling and documentation
4. **Week 4**: Type safety and architectural improvements

### 6. **Metrics to Track**

- Frame time consistency
- Memory usage patterns
- System execution time
- Clone operation frequency
- Debug build performance vs release

This plan focuses on maintainability for junior developers while achieving significant performance improvements. 