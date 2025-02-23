# Use drag_stopped() instead of drag_released()

The method `drag_released()` is deprecated in egui. Always use `drag_stopped()` instead.

Example:

```rust
// ❌ Don't use
if response.drag_released() {
    // handle drag end
}

// ✅ Do use
if response.drag_stopped() {
    // handle drag end
}
```

This applies to all egui Response objects. The `drag_stopped()` method is the newer, preferred way to detect when a drag operation has ended.
