pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod vm;

use bytecode::Disassembler;
use compiler::Compiler;
use diagnostics::Diagnostic;
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use std::io::Write;
use vm::VM;

pub fn check(source: &str, file_name: Option<&str>) -> Result<(), Vec<String>> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    let program = Parser::new(tokens)
        .parse()
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    SemanticAnalyzer::new()
        .analyze(&program)
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    Ok(())
}

pub fn compile_source(
    source: &str,
    file_name: Option<&str>,
) -> Result<bytecode::Chunk, Vec<String>> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    let program = Parser::new(tokens)
        .parse()
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    SemanticAnalyzer::new()
        .analyze(&program)
        .map_err(|diags| format_diagnostics(diags, source, file_name))?;

    Compiler::new()
        .compile(&program)
        .map_err(|diags| format_diagnostics(diags, source, file_name))
}

pub fn disassemble_source(source: &str, file_name: Option<&str>) -> Result<String, Vec<String>> {
    let chunk = compile_source(source, file_name)?;
    let display_name = file_name.unwrap_or("<main>");
    Ok(Disassembler::disassemble(&chunk, display_name))
}

pub fn run_source(
    source: &str,
    file_name: Option<&str>,
    output: &mut dyn Write,
) -> Result<(), Vec<String>> {
    let chunk = compile_source(source, file_name)?;
    let mut vm = VM::new(chunk).with_output(output);
    vm.run().map_err(|err| {
        let file = file_name.unwrap_or("<input>");
        vec![format!("{}:{}: error: {}", file, err.line, err.message)]
    })
}

fn format_diagnostics(
    diags: Vec<Diagnostic>,
    source: &str,
    file_name: Option<&str>,
) -> Vec<String> {
    diags
        .into_iter()
        .map(|d| {
            let with_f = if let Some(f) = file_name {
                d.with_file(f)
            } else {
                d
            };
            with_f.render(Some(source))
        })
        .collect()
}
