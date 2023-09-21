use super::scanner::Scanner;
use std::fs;
use std::io;
use std::io::Write;
pub struct Interpreter;

impl Interpreter {
    pub fn build() -> Self {
        Interpreter
    }

    pub fn start(&self, args: Vec<String>) {
        if args.len() == 2 {
            self.run_file(&args[1]);
        } else {
            self.run_prompt();
        }
    }

    fn run(&self, contents: String) {
        let mut scanner = Scanner::build(contents);
        let tokens = scanner.scan_tokens();
        println!("Tokens: {:?}", tokens);
    }

    fn run_file(&self, file: &str) {
        let contents = fs::read_to_string(file).expect("Unable to read file.");
        self.run(contents);
    }

    fn run_prompt(&self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Unable to read line");
            if buffer.len() <= 1 {
                break;
            } else {
                self.run(buffer);
            }
        }
    }
}
