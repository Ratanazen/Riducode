#[test]
fn test_immutable_reassignment_diagnostic() {
    let code = "val pi = 3.14\npi = 3.14159";
    let res = ridu::check(code, Some("pi.ridu"));
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("cannot reassign immutable variable 'pi' declared with 'val'"));
}

#[test]
fn test_undeclared_variable_diagnostic() {
    let code = "println(unknown_var)";
    let res = ridu::check(code, Some("err.ridu"));
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs[0].contains("cannot find variable 'unknown_var' in this scope"));
}

#[test]
fn test_duplicate_declaration_in_same_scope() {
    let code = "val a = 1\nval a = 2";
    let res = ridu::check(code, Some("dup.ridu"));
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs[0].contains("variable 'a' is already declared in this scope"));
}

#[test]
fn test_type_mismatch_on_declaration() {
    let code = "val num: Int = \"not an integer\"";
    let res = ridu::check(code, Some("type_err.ridu"));
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs[0].contains("type mismatch for variable 'num'"));
}
