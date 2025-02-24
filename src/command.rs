use crate::stroke::Stroke;

#[derive(Clone)]
pub enum Command {
    AddStroke(Stroke),
}

pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, command: Command) {
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Clear redo stack when new command is executed
    }

    pub fn undo(&mut self) -> Option<Command> {
        if let Some(command) = self.undo_stack.pop() {
            self.redo_stack.push(command.clone());
            Some(command)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Command> {
        if let Some(command) = self.redo_stack.pop() {
            self.undo_stack.push(command.clone());
            Some(command)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
} 