//! Rust bindings to the M1 tree-sitter grammar.
//!
//! ```no_run
//! let mut parser = tree_sitter::Parser::new();
//! parser.set_language(&tree_sitter_m1::LANGUAGE.into()).unwrap();
//! ```

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_m1() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for the M1 grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_m1) };

/// Syntax-highlighting query source (`queries/highlights.scm`).
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");
/// Indentation query source (`queries/indents.scm`).
pub const INDENTS_QUERY: &str = include_str!("../../queries/indents.scm");
/// Folding query source (`queries/folds.scm`).
pub const FOLDS_QUERY: &str = include_str!("../../queries/folds.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading M1 grammar");
    }
}
