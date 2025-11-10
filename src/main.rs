mod lexer;
mod parser;
mod native;
mod code_gen;

use std::fs;
use std::path::{Path, PathBuf};
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::code_gen::statements::codegen_statements;
use crate::code_gen::builder::CodeGenContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new("./examples/hello_adan.adn");
    let source = fs::read_to_string(&input_path)?;
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