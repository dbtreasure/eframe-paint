# Element Design Guidelines

## Core Principles

1. **Trait-Based Interface**: All element operations go through the `Element` trait, not direct variant access.
2. **Factory Creation**: Elements are created only through the factory functions, not direct struct instantiation.
3. **Ownership Transfer**: Element mutation uses take/modify/add pattern, avoiding reference juggling.
4. **Encapsulation**: Element implementation details stay within their respective modules.

## Element Creation

Always use the factory module:

```rust
// CORRECT
let element = element::factory::create_stroke(
    id, 
    points,
    thickness, 
    color
);

// INCORRECT - direct variant creation
let element = ElementType::Stroke(stroke::Stroke::new(
    id, 
    points, 
    thickness, 
    color
));
```

## Element Modification

Use the ownership transfer pattern:

```rust
// CORRECT
let mut element = editor_model.take_element_by_id(id)?;
element.translate(delta)?;
editor_model.add_element(element);

// INCORRECT - direct mutation through reference
if let Some(elem_mut) = editor_model.get_element_mut(id) {
    if let ElementType::Stroke(s) = elem_mut {
        s.translate_in_place(delta);
    }
}
```

## Type Checking

Use trait methods, not pattern matching:

```rust
// CORRECT
if element.element_type() == "stroke" {
    // Stroke-specific behavior
}

// INCORRECT - breaks encapsulation
if let ElementType::Stroke(_) = element {
    // Stroke-specific behavior
}
```

## Command Implementation

Commands should:

1. Use trait methods, not check types
2. Transfer ownership when modifying elements
3. Be reversible through undo/redo
4. Not leak implementation details

```rust
// CORRECT
pub fn execute(&self, model: &mut EditorModel) {
    // Take ownership from model
    let mut element = model.take_element_by_id(self.element_id)?;
    
    // Modify element
    element.translate(self.delta)?;
    
    // Return ownership to model
    model.add_element(element);
}
```

## Performance Considerations

1. **Minimize Cloning**: Use ownership transfer to avoid unnecessary clones
2. **Lazy Texturing**: Generate textures only when needed
3. **Invalidation**: Mark textures as invalid when elements change
4. **Chain Operations**: Group related operations to avoid multiple ownership transfers

## Testing

Tests should verify:

1. **Trait Compliance**: All elements correctly implement trait methods
2. **Ownership Transfer**: Elements can be taken, modified, and readded properly
3. **Command Operations**: Commands correctly modify elements and support undo
4. **Error Handling**: Invalid operations (too small, empty, etc.) are rejected

## Future Considerations

For future element types (like Text):

1. Implement the Element trait
2. Add to the ElementType enum
3. Add factory methods
4. Update all implementations of match ElementType { ... } (should be minimal)