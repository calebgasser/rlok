pub struct ErrorHandler {
    has_error: bool,
}

impl ErrorHandler {
    pub fn build() -> Self {
        ErrorHandler { has_error: false }
    }

    pub fn error(&mut self, line: i32, message: &str) {
        self.has_error = true;
        self.report(line, String::from(""), message);
    }

    fn report(&self, line: i32, location: String, message: &str) {
        println!("[line {line}] Error {location}: {message}");
    }
}
