mod lexer;
mod parser;
mod native;
mod code_gen;

use dirs;
use std::fs;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::code_gen::statements::codegen_statements;
use crate::code_gen::builder::CodeGenContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = dirs::home_dir().unwrap().join("Projects/adan/examples/hello_adan.adn");
    let source = fs::read_to_string(path)?;
    let tokens = Lexer::new(&source).tokenize()?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let context = inkwell::context::Context::create();
    let mut ctx = CodeGenContext::new(&context, "adan_module");
    for stmt in &statements {
        codegen_statements(&mut ctx, stmt)?;
    }

    ctx.module.print_to_stderr();
    Ok(())
}

