//! Rust bindings to the M1 tree-sitter grammar.
//!
//! ```no_run
//! let mut parser = tree_sitter::Parser::new();
//! parser.set_language(&tree_sitter_m1::LANGUAGE.into()).unwrap();
//! ```

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
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
/// Scope/definition/reference query source (`queries/locals.scm`).
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
/// Language-injection query source (`queries/injections.scm`).
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
/// Text-objects query source (`queries/textobjects.scm`), following the
/// nvim-treesitter-textobjects capture conventions.
pub const TEXTOBJECTS_QUERY: &str = include_str!("../../queries/textobjects.scm");
/// The grammar's generated node-types description (`src/node-types.json`),
/// consumed by downstream codegen (e.g. m1-core's `Kind` generator).
pub const NODE_TYPES_JSON: &str = include_str!("../../src/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading M1 grammar");
    }

    /// Every operator the M1 Build Development Manual lists must parse without an
    /// ERROR/MISSING node — including unary `~` and the bitwise compound
    /// assignments that were previously missing (#31).
    #[test]
    fn manual_operators_parse_without_errors() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        for src in [
            "x = ~y;\n",
            "x %= 2;\n",
            "x &= 2;\n",
            "x |= 2;\n",
            "x ^= 2;\n",
            "x <<= 2;\n",
            "x >>= 2;\n",
        ] {
            let tree = parser.parse(src, None).unwrap();
            assert!(
                !tree.root_node().has_error(),
                "expected `{src:?}` to parse without errors"
            );
        }
    }

    #[test]
    fn node_types_json_is_present() {
        assert!(super::NODE_TYPES_JSON.trim_start().starts_with('['));
        assert!(super::NODE_TYPES_JSON.contains("\"identifier\""));
    }

    #[test]
    fn queries_compile_against_grammar() {
        let language: tree_sitter::Language = super::LANGUAGE.into();
        for (name, src) in [
            ("highlights", super::HIGHLIGHTS_QUERY),
            ("indents", super::INDENTS_QUERY),
            ("folds", super::FOLDS_QUERY),
            ("locals", super::LOCALS_QUERY),
            ("textobjects", super::TEXTOBJECTS_QUERY),
        ] {
            tree_sitter::Query::new(&language, src)
                .unwrap_or_else(|e| panic!("query {name} failed to compile: {e}"));
        }
        // injections.scm is an empty placeholder; an empty query is still valid.
        tree_sitter::Query::new(&language, super::INJECTIONS_QUERY)
            .expect("injections query failed to compile");
    }

    #[test]
    fn local_declaration_exposes_type_annotation_field() {
        // Regression for #18: the type annotation must be reachable as a named
        // field on `local_declaration`, not only positionally.
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        let src = "local <Integer> foo = 1;\n";
        let tree = parser.parse(src, None).unwrap();
        let root = tree.root_node();
        let decl = root.child(0).expect("local_declaration");
        assert_eq!(decl.kind(), "local_declaration");
        let annot = decl
            .child_by_field_name("type_annotation")
            .expect("type_annotation field should be present");
        assert_eq!(annot.kind(), "type_annotation");
        // The inner `type` field still resolves to the type identifier.
        let ty = annot
            .child_by_field_name("type")
            .expect("type field on type_annotation");
        assert_eq!(&src[ty.byte_range()], "Integer");
    }

    /// Regression for #35: an unterminated `$(` interpolation must fail fast, not
    /// scan to end-of-input. Before the scanner bound, tree-sitter re-ran the
    /// external scanner at successive positions during error recovery and each of
    /// N unterminated `$(` walked to EOF — O(n^2) parse time (n=50000 took ~20s).
    /// The bound makes it linear; a generous wall-clock ceiling pins the fix
    /// without being flaky on slow CI (the post-fix parse is a few ms).
    #[test]
    fn unterminated_interpolation_does_not_blow_up() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        let src = format!("x = {};\n", "$(".repeat(50_000));
        let start = std::time::Instant::now();
        let tree = parser.parse(&src, None).unwrap();
        let elapsed = start.elapsed();
        // The `$(` never close, so it is a parse error — but it must error *fast*.
        assert!(tree.root_node().has_error());
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "unterminated `$(` x50000 took {elapsed:?}; expected <5s (O(n) parse). \
             A regression to O(n^2) re-introduces the scanner DoS (#35)."
        );
    }

    /// The interpolation scan bound must not reject legitimate interpolations,
    /// which are short and single-line.
    #[test]
    fn well_formed_interpolation_still_parses() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        let tree = parser.parse("x = $(SEG) + 1;\n", None).unwrap();
        assert!(!tree.root_node().has_error());
    }

    /// Walk the tree and return the first node of `kind` (depth-first).
    fn first_node_of_kind<'t>(
        root: tree_sitter::Node<'t>,
        kind: &str,
    ) -> Option<tree_sitter::Node<'t>> {
        let mut cursor = root.walk();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                return Some(node);
            }
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
        None
    }

    /// Regression: on CRLF (`\r\n`) line endings a `line_comment` must stop
    /// before the carriage return, not absorb it. The token rule runs to the
    /// end of the line, so without excluding `\r` the comment span ends one
    /// byte past the visible text and that stray `\r` propagates downstream
    /// (m1-fmt re-emits comment text, m1-lint counts the byte, highlighting
    /// extends onto the line terminator). LF-only corpora never tripped it.
    #[test]
    fn line_comment_excludes_trailing_carriage_return() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        let src = "// foo\r\nx = 1;\r\n";
        let tree = parser.parse(src, None).unwrap();
        let comment =
            first_node_of_kind(tree.root_node(), "line_comment").expect("a line_comment node");
        let text = &src[comment.byte_range()];
        assert_eq!(
            text, "// foo",
            "line_comment span must stop before the CRLF carriage return, got {text:?}"
        );
        assert!(
            !text.ends_with('\r'),
            "line_comment must not absorb the trailing \\r on CRLF lines"
        );
    }
}
