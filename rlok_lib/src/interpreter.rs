use super::parser::Parser;
use super::scanner::Scanner;
use color_eyre::eyre::Result;
use std::fs;
use std::io;
use std::io::Write;

pub struct Interpreter;

impl Interpreter {
    pub fn build() -> Self {
        Interpreter
    }

    pub fn start(&self, args: Vec<String>) -> Result<()> {
        if args.len() == 2 {
            self.run_file(&args[1])?;
        } else {
            self.run_prompt()?;
        }
        Ok(())
    }

    fn run(&self, contents: String) -> Result<()> {
        let mut scanner = Scanner::build(contents);
        let tokens = scanner.scan_tokens()?;
        println!("Tokens: {:?}", tokens);
        let mut parser = Parser::new(tokens)?;
        if let Some(ast) = parser.parse()? {
            println!("Ast: {}", ast);
        } else {
            eprintln!("Parser returned none...")
        }
        Ok(())
    }

    fn run_file(&self, file: &str) -> Result<()> {
        let contents = fs::read_to_string(file)?;
        self.run(contents)?;
        Ok(())
    }

    fn run_prompt(&self) -> Result<()> {
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
                match self.run(buffer) {
                    Ok(()) => (),
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
        Ok(())
    }
}
