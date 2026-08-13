use std::sync::Mutex;

pub struct ErrorLog {
    errors: Mutex<Vec<String>>,
}

impl ErrorLog {
    pub fn log(&self, error: String) {
        let mut last_error = self.errors.lock().unwrap();
        last_error.push(error);
    }

    pub fn all_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    pub fn last_error(&self) -> Option<String> {
        self.errors.lock().unwrap().iter().last().cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.lock().unwrap().is_empty()
    }
}

impl Default for ErrorLog {
    fn default() -> Self {
        Self {
            errors: Default::default(),
        }
    }
}
