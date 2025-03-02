# Glossary of Terms

This document provides definitions for key terms used in the tool state management system.

## Core Concepts

### Tool

An interactive component that allows users to manipulate the document. Tools implement the `Tool` trait and can be in various states.

### Tool State

A specific configuration or mode of a tool, represented as a type parameter (e.g., `DrawStrokeTool<Ready>`, `SelectionTool<Scaling>`).

### State Transition

The process of changing a tool from one state to another, enforced by Rust's type system.

### Tool Pool

A cache of tool instances that can be reused to avoid allocations during tool transitions.

### State Retention

The preservation of tool configuration between activations.

### Version Tracking

A mechanism for detecting state changes using monotonically increasing counters.

## Data Structures

### ToolType

An enum containing all possible tool types and states:

```rust
pub enum ToolType {
    DrawStroke(DrawStrokeToolType),
    Selection(SelectionToolType),
    // Add more tools here as they are implemented
}
```

### DrawStrokeToolType

An enum containing the possible states of the drawing tool:

```rust
pub enum DrawStrokeToolType {
    Ready(DrawStrokeTool<Ready>),
    Drawing(DrawStrokeTool<Drawing>),
}
```

### SelectionToolType

An enum containing the possible states of the selection tool:

```rust
pub enum SelectionToolType {
    Active(SelectionTool<Active>),
    TextureSelected(SelectionTool<TextureSelected>),
    ScalingEnabled(SelectionTool<ScalingEnabled>),
    Scaling(SelectionTool<Scaling>),
}
```

### ToolPool

A structure that caches tool instances for reuse:

```rust
pub struct ToolPool {
    selection_tool: Option<SelectionToolType>,
    draw_stroke_tool: Option<DrawStrokeToolType>,
    retained_states: HashMap<&'static str, ToolType>,
}
```

### EditorState

A structure that holds the current state of the editor, including the active tool:

```rust
pub struct EditorState {
    shared: Arc<EditorStateData>,
}
```

### EditorStateData

The inner data structure of EditorState:

```rust
struct EditorStateData {
    active_tool: Option<Arc<ToolType>>,
    selected_elements: Arc<[ElementType]>,
    version: u64,
}
```

### EditorStateBuilder

A builder pattern implementation for creating new EditorState instances:

```rust
pub struct EditorStateBuilder {
    data: EditorStateData,
}
```

## Error Types

### TransitionError

An enum representing errors that can occur during tool state transitions:

```rust
pub enum TransitionError {
    InvalidStateTransition { from: &'static str, to: &'static str, state: String },
    ToolBusy(String),
    MemorySafetyViolation,
}
```

## Tool States

### DrawStrokeTool States

#### Ready

The default state of the drawing tool, ready to start drawing.

#### Drawing

The active drawing state, where the tool is creating a stroke.

### SelectionTool States

#### Active

The default state of the selection tool, ready to select elements.

#### TextureSelected

A state where a texture (image) is selected and can be manipulated.

#### ScalingEnabled

A state where scaling is enabled but not yet active.

#### Scaling

The active scaling state, where the tool is resizing an element.

## Methods and Functions

### can_transition

A method that checks if a tool can transition to a new state:

```rust
pub fn can_transition(&self) -> bool
```

### validate_transition

A method that validates a transition between states:

```rust
pub fn validate_transition(&self, current_state: &str, tool: &ToolType) -> Result<bool, TransitionError>
```

### restore_state

A method that restores a tool's state from another instance:

```rust
pub fn restore_state(&mut self, other: &Self)
```

### retain_state

A method that stores a tool's state for later restoration:

```rust
pub fn retain_state(&mut self, tool: ToolType)
```

### get

A method that retrieves a tool from the pool:

```rust
pub fn get(&mut self, tool_name: &str) -> Option<ToolType>
```

### return_tool

A method that returns a tool to the pool:

```rust
pub fn return_tool(&mut self, tool: ToolType)
```

### version

A method that returns the current version of the editor state:

```rust
pub fn version(&self) -> u64
```

### builder

A method that creates a builder for modifying the editor state:

```rust
pub fn builder(&self) -> EditorStateBuilder
```

## Performance Terms

### Allocation

The process of reserving memory for a new object.

### Arc (Atomic Reference Counting)

A thread-safe reference-counted pointer type that enables efficient sharing of data.

### Clone

Creating a copy of an object, which is cheap for Arc-wrapped data.

### Deep Copy

Creating a complete copy of an object and all its contents.

### Shallow Copy

Creating a copy of an object that shares some of its contents with the original.

### Reference Cycle

A situation where objects reference each other in a cycle, preventing automatic cleanup.

### Weak Reference

A non-owning reference that doesn't prevent an object from being dropped.

## Design Patterns

### Type State Pattern

A design pattern that uses Rust's type system to enforce state transitions.

### Builder Pattern

A design pattern that separates the construction of a complex object from its representation.

### State Pattern

A behavioral design pattern that allows an object to alter its behavior when its internal state changes.

### Finite State Machine

A computational model that can be in exactly one of a finite number of states at any given time.

### Immutable State

A pattern where state changes produce new state instances rather than mutating existing ones.
