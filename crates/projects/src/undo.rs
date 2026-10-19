use iris_core::IrisResult;
use iris_timeline::{Command, Timeline};

pub struct UndoRedoManager {
    undo_stack: Vec<Box<dyn Command + Send + Sync>>,
    redo_stack: Vec<Box<dyn Command + Send + Sync>>,
    max_history: usize,
}

impl Default for UndoRedoManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl UndoRedoManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: max_history.max(1),
        }
    }

    pub fn execute_command(
        &mut self,
        command: Box<dyn Command + Send + Sync>,
        timeline: &mut Timeline,
    ) -> IrisResult<()> {
        command.execute(timeline)?;
        self.undo_stack.push(command);
        self.redo_stack.clear();

        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    pub fn undo(&mut self, timeline: &mut Timeline) -> IrisResult<bool> {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.undo(timeline)?;
            self.redo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self, timeline: &mut Timeline) -> IrisResult<bool> {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.execute(timeline)?;
            self.undo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}
