//! Fuzz incremental parsing: parse, apply an arbitrary edit, re-parse with the
//! old tree, and assert the incremental result matches a fresh parse (#46) —
//! the invariant every editor integration (m1-lsp, nvim, VS Code) relies on.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (String, u16, u16, String)| {
    let (src, start, len, insert) = input;
    if src.len() > 4096 || insert.len() > 64 {
        return; // keep iterations fast; long inputs add no new coverage here
    }

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_m1::LANGUAGE.into())
        .unwrap();
    let Some(mut tree) = parser.parse(&src, None) else {
        return;
    };

    // Snap the fuzzed edit to char boundaries inside the document.
    let snap = |b: usize| -> usize {
        let mut b = b.min(src.len());
        while !src.is_char_boundary(b) {
            b -= 1;
        }
        b
    };
    let start = snap(start as usize);
    let end = snap(start + len as usize);

    let mut edited = String::with_capacity(src.len() + insert.len());
    edited.push_str(&src[..start]);
    edited.push_str(&insert);
    edited.push_str(&src[end..]);

    let point_at = |s: &str, byte: usize| -> tree_sitter::Point {
        let before = &s[..byte];
        let row = before.matches('\n').count();
        let col = before.len() - before.rfind('\n').map_or(0, |i| i + 1);
        tree_sitter::Point::new(row, col)
    };
    tree.edit(&tree_sitter::InputEdit {
        start_byte: start,
        old_end_byte: end,
        new_end_byte: start + insert.len(),
        start_position: point_at(&src, start),
        old_end_position: point_at(&src, end),
        new_end_position: point_at(&edited, start + insert.len()),
    });

    let incremental = parser.parse(&edited, Some(&tree)).unwrap();
    let fresh = parser.parse(&edited, None).unwrap();
    assert_eq!(
        incremental.root_node().to_sexp(),
        fresh.root_node().to_sexp(),
        "incremental parse diverged from fresh parse"
    );
});
