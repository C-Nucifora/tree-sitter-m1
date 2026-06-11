//! Fuzz the parser + external scanner over arbitrary bytes: must never crash,
//! abort, or hang (#46). The scanner (src/scanner.c) is hand-written C handling
//! space-joined identifier segments, digit-led words and `$(…)` interpolation —
//! the toolchain's rawest input surface.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_m1::LANGUAGE.into())
        .unwrap();
    // A parse may legitimately produce ERROR nodes; it must simply return.
    let _tree = parser.parse(src, None);
});
