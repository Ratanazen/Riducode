use std::env;
use std::fs;
use std::io;
use std::process;

const VERSION: &str = "0.1.0";

fn print_help() {
    println!("Ridu Programming Language Toolchain v{}", VERSION);
    println!("Usage: ridu <command> [options] <file.ridu>\n");
    println!("Commands:");
    println!("  run <file>       Compile and execute a Ridu program on the virtual machine");
    println!("  check <file>     Validate syntax and semantics without running");
    println!("  disasm <file>    Compile and disassemble the Ridu bytecode chunk");
    println!("  help             Show this help information");
    println!("  version          Show version information\n");
    println!("Options:");
    println!("  -h, --help       Show help");
    println!("  -V, --version    Show version");
}

fn print_version() {
    println!("ridu v{} (Phase 1 Compiler MVP)", VERSION);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(1);
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
        }
        "-V" | "--version" | "version" => {
            print_version();
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for 'run'");
                eprintln!("Usage: ridu run <file.ridu>");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    process::exit(1);
                }
            };

            let mut stdout = io::stdout();
            if let Err(errors) = ridu::run_source(&source, Some(file_path), &mut stdout) {
                for err in errors {
                    eprint!("{}", err);
                }
                process::exit(1);
            }
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for 'check'");
                eprintln!("Usage: ridu check <file.ridu>");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    process::exit(1);
                }
            };

            match ridu::check(&source, Some(file_path)) {
                Ok(()) => {
                    println!("Validation successful: '{}' has no errors.", file_path);
                }
                Err(errors) => {
                    for err in errors {
                        eprint!("{}", err);
                    }
                    process::exit(1);
                }
            }
        }
        "disasm" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for 'disasm'");
                eprintln!("Usage: ridu disasm <file.ridu>");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    process::exit(1);
                }
            };

            match ridu::disassemble_source(&source, Some(file_path)) {
                Ok(disasm) => {
                    print!("{}", disasm);
                }
                Err(errors) => {
                    for err in errors {
                        eprint!("{}", err);
                    }
                    process::exit(1);
                }
            }
        }
        unknown => {
            eprintln!("Unknown command: '{}'", unknown);
            print_help();
            process::exit(1);
        }
    }
}
