use std::sync::Mutex;

pub struct ErrorLog {
    last_error: Mutex<Option<String>>,
}

impl ErrorLog {
    pub fn log(&self, error: String) {
        let mut last_error = self.last_error.lock().unwrap();
        *last_error = Some(error);
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().clone()
    }
}

impl Default for ErrorLog {
    fn default() -> Self {
        Self {
            last_error: Default::default(),
        }
    }
}
