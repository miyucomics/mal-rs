use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};
use std::cell::RefCell;

thread_local! {
    static EDITOR : RefCell<Wrapper> = {
        let mut editor = Editor::new().unwrap();
        let _ = editor.load_history(".history");
        RefCell::new(Wrapper { editor })
    };
}

// just exists so history can automatically be saved when the editor is dropped
struct Wrapper {
    editor: Editor<(), DefaultHistory>,
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        let _ = self.editor.save_history(".history");
    }
}

pub fn readline(prompt: &str) -> Option<String> {
    EDITOR.with_borrow_mut(|wrapper| match wrapper.editor.readline(prompt) {
        Ok(line) => {
            let line = line.trim();
            if !line.is_empty() {
                let _ = wrapper.editor.add_history_entry(line);
            }
            Some(line.to_string())
        }
        Err(ReadlineError::Eof) => None,
        Err(ReadlineError::Interrupted) => None,
        Err(_) => panic!("unrecoverable error reading input"),
    })
}
