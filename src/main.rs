use colored::Colorize;
use std::{
    fs,
    io::{self, Write},
};

use crate::{
    frontend::{lexer, parser},
    runtime::values,
};

mod frontend;
mod runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repl = true;

    if repl {
        println!("STARTING LANGUAGE REPL");
        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut src = String::new();
            io::stdin().read_line(&mut src)?;

            let src = src.trim();

            if src == "exit()" || src == "quit()" {
                break;
            }

            if src.is_empty() {
                continue;
            }

            let mut lexer = lexer::Lexer::new(src.to_string());
            let tokens = lexer.tokenize();

            let mut parser = parser::Parser::new(tokens);

            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(err) => {
                    eprintln!("{}", format!("Frontend error: {:?}", err).red());
                    continue;
                }
            };

            let mut inter = values::Interpreter::new();

            match inter.eval_program(&ast) {
                Ok(value) => println!("{}", format!("{:?}", value).bright_green()),
                Err(err) => eprintln!("{}", format!("Runtime error: {:?}", err).red()),
            }
        }
    } else {
        let src = fs::read_to_string("example.linguo")?;

        let mut lexer = lexer::Lexer::new(src);
        let tokens = lexer.tokenize();

        let mut parser = parser::Parser::new(tokens);
        let ast = parser.parse()?;

        println!("AST: {:#?}", ast);

        let mut inter = values::Interpreter::new();
        let value = inter.eval_program(&ast)?;

        println!("value: {:?}", value);
    }

    Ok(())
}
