use std::fs;

use crate::{
    frontend::{lexer, parser},
    runtime::values,
};

mod frontend;
mod runtime;

use std::io::{self, Write};

fn main() {
    let repl = true;

    if repl {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut src = String::new();
            io::stdin().read_line(&mut src).unwrap();

            let src = src.trim();

            if src == "exit" || src == "quit" {
                break;
            }

            if src.is_empty() {
                continue;
            }

            let mut lexer = lexer::Lexer::new(src.to_string());
            let tokens = lexer.tokenize();

            let mut parser = parser::Parser::new(tokens);
            let ast = parser.parse();
            println!("value: {:#?}", ast);

            let mut inter = values::Interpreter::new();
            let values = inter.eval_program(ast);
            println!("value: {:?}", values);
        }
    } else {
        let src = fs::read_to_string("example.linguo").expect("boom");

        let mut lexer = lexer::Lexer::new(src);
        let tokens = lexer.tokenize();

        let mut parser = parser::Parser::new(tokens);
        let ast = parser.parse();

        let mut inter = values::Interpreter::new();
        let value = inter.eval_program(ast);

        println!("{:#?}", value);
    }
}
