# eframe-paint Implementation Considerations

## Element ID Management

- Stroke IDs are currently derived from Arc pointer addresses, which change when strokes are modified
- Consider implementing a stable ID system for strokes similar to images
- This would improve selection persistence and command targeting

## Stroke Manipulation

- Stroke translation creates new strokes with translated points
- Consider adding more efficient in-place modification for performance
- Investigate ways to maintain selection when strokes are modified

## Resize Operations

- Stroke resizing requires special handling to maintain stroke appearance
- Consider implementing proportional scaling of stroke thickness
- Add support for non-uniform scaling (different X/Y scaling factors)

## Hit Testing

- Current hit testing for strokes checks proximity to line segments
- Consider more sophisticated hit testing for complex strokes
- Investigate spatial partitioning for performance with many elements

## Undo/Redo

- Current undo/redo for strokes is limited
- Consider storing original state for more robust undo operations
- Implement proper undo for resize operations
