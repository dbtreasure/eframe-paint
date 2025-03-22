use std::sync::atomic::{AtomicUsize, Ordering};

// Single static counter for all elements
static NEXT_ELEMENT_ID: AtomicUsize = AtomicUsize::new(1);

pub fn generate_id() -> usize {
    NEXT_ELEMENT_ID.fetch_add(1, Ordering::SeqCst)
}
