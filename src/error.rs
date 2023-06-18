pub struct LoxError {
    pub has_error: bool
}

impl LoxError {
    pub fn new() -> Self {
        Self {
            has_error: false
        }
    }
    pub fn error(&self, line: i32, message: String) {
        LoxError::report(line, String::new(), message);
    }

    fn report(line: i32, location: String, message: String) {
        eprintln!("[line \"{line}\"] Error {location}: {message}");

    }
}
