use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use thiserror::Error;
use super::{EditorState, EditorContext};
use crate::document::Document;
use crate::tool::ToolType;
use crate::util::time;

/// Errors that can occur during state persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Failed to serialize state: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Failed to write state: {0}")]
    WriteError(#[from] std::io::Error),
    
    #[error("Failed to read state file: {0}")]
    ReadError(String),
    
    #[error("Invalid state data: {0}")]
    InvalidState(String),
}

/// Result type for persistence operations
pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Represents a snapshot of the editor state that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSnapshot {
    /// The editor state
    pub state: EditorState,
    /// The document state
    pub document: Document,
    /// The current tool
    pub current_tool: ToolType,
    /// Timestamp of when the snapshot was taken
    pub timestamp: u64,
    /// Version of the application when the snapshot was taken
    pub version: String,
}

impl EditorSnapshot {
    /// Create a new snapshot from the current editor context
    pub fn new(ctx: &EditorContext) -> Self {
        Self {
            state: ctx.state.clone(),
            document: ctx.document.clone(),
            current_tool: ctx.current_tool.clone(),
            timestamp: time::timestamp_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Restore this snapshot to an editor context
    pub fn restore(self, ctx: &mut EditorContext) -> PersistenceResult<()> {
        // Validate version compatibility
        if self.version != env!("CARGO_PKG_VERSION") {
            // For now, just log a warning for version mismatch
            eprintln!("Warning: Snapshot version {} differs from current version {}", 
                self.version, env!("CARGO_PKG_VERSION"));
        }

        // Restore state
        ctx.state = self.state;
        ctx.document = self.document;
        ctx.current_tool = self.current_tool;

        // Emit state changed event
        ctx.event_bus.emit(crate::event::EditorEvent::StateChanged {
            old: EditorState::Idle, // We don't know the true old state
            new: ctx.state.clone(),
        });

        Ok(())
    }
}

/// Manages state persistence and recovery
#[derive(Debug, Clone)]
pub struct StatePersistence {
    /// Directory where state files are stored
    state_dir: String,
    /// Maximum number of auto-save files to keep
    max_autosaves: usize,
    /// Interval between auto-saves in seconds
    autosave_interval: u64,
    /// Last auto-save timestamp
    last_autosave: u64,
}

impl StatePersistence {
    /// Create a new state persistence manager
    pub fn new(state_dir: String) -> Self {
        Self {
            state_dir,
            max_autosaves: 5,
            autosave_interval: 300, // 5 minutes
            last_autosave: 0,
        }
    }

    /// Save a snapshot of the current editor state
    pub fn save_snapshot(&self, ctx: &EditorContext, name: &str) -> PersistenceResult<()> {
        let snapshot = EditorSnapshot::new(ctx);
        let path = Path::new(&self.state_dir).join(format!("{}.json", name));

        // Create state directory if it doesn't exist
        fs::create_dir_all(&self.state_dir)?;

        // Serialize and save
        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Load a snapshot by name
    pub fn load_snapshot(&self, name: &str) -> PersistenceResult<EditorSnapshot> {
        let path = Path::new(&self.state_dir).join(format!("{}.json", name));
        let json = fs::read_to_string(path)
            .map_err(|e| PersistenceError::ReadError(e.to_string()))?;
        
        serde_json::from_str(&json)
            .map_err(|e| PersistenceError::SerializationError(e))
    }

    /// Check if we should auto-save based on the interval
    pub fn should_autosave(&self) -> bool {
        let now = time::timestamp_secs();
        now - self.last_autosave >= self.autosave_interval
    }

    /// Perform auto-save if needed
    pub fn try_autosave(&mut self, ctx: &EditorContext) -> PersistenceResult<()> {
        if self.should_autosave() {
            let now = time::timestamp_secs();
            
            // Save with timestamp
            self.save_snapshot(ctx, &format!("autosave_{}", now))?;
            self.last_autosave = now;

            // Cleanup old autosaves
            self.cleanup_old_autosaves()?;
        }
        Ok(())
    }

    /// Clean up old auto-save files
    fn cleanup_old_autosaves(&self) -> PersistenceResult<()> {
        let dir = Path::new(&self.state_dir);
        let mut autosaves: Vec<_> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_string_lossy()
                    .starts_with("autosave_")
            })
            .collect();

        // Sort by modification time
        autosaves.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
        });

        // Remove oldest files if we have too many
        while autosaves.len() > self.max_autosaves {
            if let Some(oldest) = autosaves.first() {
                fs::remove_file(oldest.path())?;
                autosaves.remove(0);
            }
        }

        Ok(())
    }

    /// Find the most recent auto-save file
    pub fn find_latest_autosave(&self) -> PersistenceResult<Option<String>> {
        let dir = Path::new(&self.state_dir);
        let mut latest = None;
        let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("autosave_") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest = Some(name_str.into_owned());
                        }
                    }
                }
            }
        }

        Ok(latest)
    }
} 