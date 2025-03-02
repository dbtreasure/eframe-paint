# Troubleshooting Guide

This document provides solutions for common issues encountered with the tool state management system.

## Common Issues and Solutions

### 1. "Cannot transition" errors during tool operations

**Symptoms:**

- Error message: `Cannot transition from X to Y`
- Tool appears stuck in current state
- UI becomes unresponsive to tool changes

**Possible Causes:**

- Active operation not completed
- Transform handles not released
- Invalid state transition attempted

**Solutions:**

1. Check if the tool has pending operations:

   ```rust
   if tool.has_pending_operations() {
       // Complete or cancel the operation first
   }
   ```

2. Ensure all transform handles are released:

   ```rust
   if tool.has_active_transform() {
       // Finish or cancel the transform first
   }
   ```

3. Validate transition before attempting:

   ```rust
   if tool_pool.can_transition(&tool) {
       // Safe to transition
   } else {
       // Log error and provide user feedback
   }
   ```

4. Emergency reset (last resort):
   ```rust
   // Force tool back to its default state
   match tool {
       ToolType::DrawStroke(_) => {
           tool = ToolType::DrawStroke(new_draw_stroke_tool());
       },
       ToolType::Selection(_) => {
           tool = ToolType::Selection(new_selection_tool());
       },
   }
   ```

### 2. Tool settings reset unexpectedly

**Symptoms:**

- Color, thickness, or other settings revert to defaults
- User preferences not preserved between tool activations
- Inconsistent tool behavior

**Possible Causes:**

- Missing state retention
- Incomplete `restore_state` implementation
- Tool pool not used correctly

**Solutions:**

1. Ensure tool state is retained when deactivated:

   ```rust
   // When deactivating a tool
   tool_pool.retain_state(tool);
   ```

2. Check `restore_state` implementation for the tool:

   ```rust
   // In DrawStrokeToolType
   pub fn restore_state(&mut self, other: &Self) {
       match (self, other) {
           (Self::Ready(self_tool), Self::Ready(other_tool)) => {
               self_tool.set_color(other_tool.color());
               self_tool.set_thickness(other_tool.thickness());
           },
           // Handle other state combinations
           _ => {}
       }
   }
   ```

3. Use the tool pool for all tool transitions:

   ```rust
   // Get tool from pool
   let tool = tool_pool.get("DrawStroke").unwrap_or_else(|| ToolType::DrawStroke(new_draw_stroke_tool()));

   // Return tool to pool when done
   tool_pool.return_tool(tool);
   ```

### 3. High memory usage

**Symptoms:**

- Increasing memory consumption over time
- Performance degradation
- Application crashes

**Possible Causes:**

- Excessive retained states
- Arc reference cycles
- Missing tool pool returns

**Solutions:**

1. Check retained states count:

   ```rust
   println!("Retained states: {}", tool_pool.retained_states.len());
   ```

2. Limit retained states:

   ```rust
   // Implement a cleanup method for ToolPool
   pub fn cleanup_retained_states(&mut self) {
       // Keep only the most recently used states
       if self.retained_states.len() > 10 {
           // Remove oldest entries
       }
   }
   ```

3. Ensure tools are returned to the pool:

   ```rust
   // Always return tools to the pool when done
   tool_pool.return_tool(tool);
   ```

4. Check for Arc reference cycles:

   ```rust
   // Use weak references where appropriate
   use std::sync::Weak;

   struct Parent {
       child: Arc<Child>,
   }

   struct Child {
       parent: Weak<Parent>,
   }
   ```

### 4. Inconsistent tool behavior

**Symptoms:**

- Tools behave differently in similar situations
- Unexpected state transitions
- Operations applied incorrectly

**Possible Causes:**

- Inconsistent state handling
- Missing validation
- Incorrect tool type matching

**Solutions:**

1. Use pattern matching exhaustively:

   ```rust
   match tool {
       ToolType::DrawStroke(DrawStrokeToolType::Ready(ready_tool)) => {
           // Handle Ready state
       },
       ToolType::DrawStroke(DrawStrokeToolType::Drawing(drawing_tool)) => {
           // Handle Drawing state
       },
       ToolType::Selection(selection_tool) => {
           // Handle all selection tool states
       },
       // Add a catch-all to detect missing cases
       _ => {
           log::warn!("Unhandled tool state: {}", tool.current_state_name());
       }
   }
   ```

2. Add logging for state transitions:

   ```rust
   log::debug!("Transitioning from {} to {}",
       tool.current_state_name(),
       new_tool.current_state_name());
   ```

3. Implement consistent validation:
   ```rust
   // Before any transition
   if !tool_pool.validate_transition(tool.current_state_name(), &new_tool)? {
       return Err(TransitionError::InvalidStateTransition {
           from: tool.current_state_name(),
           to: new_tool.current_state_name(),
           state: format!("{:?}", tool),
       });
   }
   ```

### 5. Editor state not updating

**Symptoms:**

- UI doesn't reflect tool changes
- Selection appears out of sync
- Commands have no effect

**Possible Causes:**

- Missing state updates
- Version not incremented
- State not propagated to UI

**Solutions:**

1. Check version increments:

   ```rust
   let old_version = state.version();
   let new_state = state.update_tool(|_| Some(new_tool));
   assert_ne!(old_version, new_state.version());
   ```

2. Use the builder pattern correctly:

   ```rust
   // Incorrect: build() not called
   let builder = state.builder()
       .with_active_tool(Some(tool));
   // state not updated!

   // Correct
   let new_state = state.builder()
       .with_active_tool(Some(tool))
       .build();
   // Use new_state
   ```

3. Ensure state is propagated:
   ```rust
   // Update UI with new state
   ui.ctx().request_repaint();
   ```

## Debugging Techniques

### 1. State Inspection

Print the current state of tools and transitions:

```rust
println!("Tool: {}, State: {}", tool.name(), tool.current_state_name());
println!("Can transition: {}", tool_pool.can_transition(&tool));
println!("Has pending operations: {}", tool.has_pending_operations());
println!("Has active transform: {}", tool.has_active_transform());
```

### 2. Transition Logging

Add logging to track all transitions:

```rust
// Add to ToolPool::validate_transition
log::debug!("Validating transition from {} to {}: {}",
    current_state,
    tool.current_state_name(),
    result.is_ok());
```

### 3. Version Tracking

Monitor state version changes:

```rust
let old_version = state.version();
// ... perform operations ...
let new_version = state.version();
println!("Version delta: {}", new_version - old_version);
```

### 4. Tool Pool Statistics

Track tool pool usage:

```rust
// Add counters to ToolPool
pub struct ToolPool {
    // ... existing fields ...
    hits: usize,
    misses: usize,
}

// Increment in get() method
pub fn get(&mut self, tool_name: &str) -> Option<ToolType> {
    let result = match tool_name {
        "Selection" => self.selection_tool.take().map(ToolType::Selection),
        "DrawStroke" => self.draw_stroke_tool.take().map(ToolType::DrawStroke),
        _ => None,
    };

    if result.is_some() {
        self.hits += 1;
    } else {
        self.misses += 1;
    }

    result
}
```

## Maintenance Checklist

Regular maintenance tasks to prevent issues:

- [ ] Validate all state transitions weekly
- [ ] Check memory usage patterns
- [ ] Review error logs for transition failures
- [ ] Test tool pooling efficiency
- [ ] Verify state retention behavior
- [ ] Run memory safety checks with MIRI
- [ ] Benchmark performance against baseline

## Emergency Recovery

If the application becomes unresponsive due to tool state issues:

1. **Reset all tools**:

   ```rust
   // Reset all tools to their default states
   tool_pool = ToolPool::new();
   ```

2. **Clear retained states**:

   ```rust
   tool_pool.retained_states.clear();
   ```

3. **Reset editor state**:

   ```rust
   state = EditorState::new();
   ```

4. **Force UI update**:
   ```rust
   ui.ctx().request_repaint();
   ```

## Reporting Issues

When reporting tool state issues, include:

1. Current tool and state name
2. Attempted operation
3. Error message (if any)
4. Steps to reproduce
5. Version information
6. State version number
