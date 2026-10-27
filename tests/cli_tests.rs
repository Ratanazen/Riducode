use std::io::Cursor;

#[test]
fn test_hello_world_execution() {
    let code = r#"
val greeting = "Hello, Ridu!"
println(greeting)
"#;
    let mut out = Cursor::new(Vec::new());
    ridu::run_source(code, Some("hello.ridu"), &mut out).expect("execution should succeed");

    let output_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(output_str.trim(), "Hello, Ridu!");
}

#[test]
fn test_variable_mutation_and_scope() {
    let code = r#"
var count = 1
count = count + 2
{
    var count = 100
    count = count + 50
    println(count)
}
println(count)
"#;
    let mut out = Cursor::new(Vec::new());
    ridu::run_source(code, Some("scope.ridu"), &mut out).expect("execution should succeed");

    let output_str = String::from_utf8(out.into_inner()).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines, vec!["150", "3"]);
}

#[test]
fn test_arithmetic_and_comparisons() {
    let code = r#"
val a = 20
val b = 6
println(a + b)
println(a - b)
println(a * b)
println(a / b)
println(a % b)
println(a > b)
println(a == 20)
println(!false)
println(true && (b == 6))
"#;
    let mut out = Cursor::new(Vec::new());
    ridu::run_source(code, Some("arith.ridu"), &mut out).expect("execution should succeed");

    let output_str = String::from_utf8(out.into_inner()).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(
        lines,
        vec!["26", "14", "120", "3", "2", "true", "true", "true", "true"]
    );
}

#[test]
fn test_disassembler_output() {
    let code = "val x = 10 + 20\nprintln(x)";
    let disasm = ridu::disassemble_source(code, Some("test.ridu")).unwrap();
    assert!(disasm.contains("=== Bytecode Chunk: test.ridu ==="));
    assert!(disasm.contains("OP_CONSTANT"));
    assert!(disasm.contains("OP_ADD"));
    assert!(disasm.contains("OP_DEF_GLOBAL"));
    assert!(disasm.contains("OP_PRINTLN"));
    assert!(disasm.contains("OP_HALT"));
}
