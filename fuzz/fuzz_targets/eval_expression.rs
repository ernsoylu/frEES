#![no_main]

use frees_core::{eval, lexer, parser};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    if let Ok(tokens) = lexer::tokenize(&source) {
        let mut cursor = parser::Cursor::new(&tokens, &source);
        if let Ok(expr) = parser::parse_expr(&mut cursor) {
            let scope = eval::Scope::from_iter([("x".into(), 2.0), ("y".into(), -1.0)]);
            let _ = eval::eval(&expr, &scope);
        }
    }
});
