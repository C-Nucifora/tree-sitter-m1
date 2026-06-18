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

/// The tree-sitter grammar metadata (`tree-sitter.json`). Its
/// `metadata.version` is what `tree-sitter generate` embeds into the generated
/// `src/parser.c` `TSLanguageMetadata`, so it must agree with the crate version.
#[cfg(test)]
const TREE_SITTER_JSON: &str = include_str!("../../tree-sitter.json");

/// The npm package manifest (`package.json`); its `version` field is the third
/// independently-declared version string that must agree with the crate.
#[cfg(test)]
const PACKAGE_JSON: &str = include_str!("../../package.json");

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

    /// The three independently-declared version strings must agree:
    /// `Cargo.toml` (`CARGO_PKG_VERSION`), `tree-sitter.json` `metadata.version`,
    /// and `package.json` `version`. This matters because `tree-sitter generate`
    /// embeds `tree-sitter.json`'s `metadata.version` into the generated
    /// `src/parser.c` `TSLanguageMetadata`, while the release tag is driven
    /// solely by `Cargo.toml`. The CI "generated parser is fresh" gate
    /// regenerates `parser.c` *from* `tree-sitter.json`, so it can never catch a
    /// `tree-sitter.json` that is stale relative to `Cargo.toml` — the two always
    /// agree by construction. Without this assertion, a release that bumps
    /// `Cargo.toml` but forgets `tree-sitter.json`/`package.json` ships a tag
    /// whose embedded parser metadata version is the *previous* release's number,
    /// with fully green CI. Pin all three here so that divergence fails fast.
    #[test]
    fn declared_versions_agree() {
        let cargo_version = env!("CARGO_PKG_VERSION");

        let ts_json: serde_json::Value =
            serde_json::from_str(super::TREE_SITTER_JSON).expect("tree-sitter.json is valid JSON");
        let ts_version = ts_json
            .get("metadata")
            .and_then(|m| m.get("version"))
            .and_then(|v| v.as_str())
            .expect("tree-sitter.json has metadata.version string");
        assert_eq!(
            ts_version, cargo_version,
            "tree-sitter.json metadata.version ({ts_version}) differs from Cargo.toml version \
             ({cargo_version}). tree-sitter generate embeds metadata.version into src/parser.c's \
             TSLanguageMetadata, but the release tag comes from Cargo.toml — they must match, and \
             the parser-freshness CI gate cannot detect this divergence."
        );

        let pkg_json: serde_json::Value =
            serde_json::from_str(super::PACKAGE_JSON).expect("package.json is valid JSON");
        let pkg_version = pkg_json
            .get("version")
            .and_then(|v| v.as_str())
            .expect("package.json has version string");
        assert_eq!(
            pkg_version, cargo_version,
            "package.json version ({pkg_version}) differs from Cargo.toml version \
             ({cargo_version}); keep all three declared versions in sync."
        );
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

    /// The set of M1 reserved keywords lives in two hand-maintained places that
    /// MUST stay in sync: the anonymous string-literal keyword tokens in
    /// `grammar.js` (surfacing as alphabetic *anonymous* node types in
    /// `node-types.json`) and the `RESERVED[]` array in `src/scanner.c`. The
    /// external scanner refuses to fold a reserved word into a space-joined
    /// identifier segment; a keyword present in the grammar but missing from
    /// `RESERVED[]` is silently swallowed into an identifier, and the construct
    /// using it parses to an ERROR node. This drift is invisible to the corpus
    /// suite (dropping a keyword from `RESERVED[]` passes every corpus test),
    /// so it is pinned here instead.
    ///
    /// `EXPECTED` is the single source of truth. It MUST match the literals in
    /// `src/scanner.c`'s `RESERVED[]` array. Updating one keyword means updating
    /// `grammar.js`, `src/scanner.c` *and* this list — the two assertions below
    /// fail if any of those drift apart.
    const EXPECTED_RESERVED_KEYWORDS: &[&str] = &[
        "local", "if", "else", "and", "or", "not", "eq", "neq", "true", "false", "static", "when",
        "is", "expand", "to",
    ];

    /// Direction A: the keyword set the *grammar* actually produces (derived from
    /// the generated `node-types.json`) must equal `EXPECTED_RESERVED_KEYWORDS`.
    /// Catches a keyword added to / removed from `grammar.js` without updating
    /// the expected list (and therefore the scanner).
    #[test]
    fn grammar_keyword_set_matches_expected() {
        let node_types: serde_json::Value =
            serde_json::from_str(super::NODE_TYPES_JSON).expect("node-types.json is valid JSON");
        let mut grammar_keywords: Vec<String> = node_types
            .as_array()
            .expect("node-types.json is a JSON array")
            .iter()
            .filter(|entry| entry.get("named").and_then(|n| n.as_bool()) == Some(false))
            .filter_map(|entry| entry.get("type").and_then(|t| t.as_str()))
            // Alphabetic anonymous node types are exactly the reserved keywords;
            // punctuation/operator tokens (`{`, `==`, `<<=`, …) are excluded.
            .filter(|t| t.chars().all(|c| c.is_ascii_alphabetic() || c == '_'))
            .map(|t| t.to_string())
            .collect();
        grammar_keywords.sort();
        grammar_keywords.dedup();

        let mut expected: Vec<String> = EXPECTED_RESERVED_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect();
        expected.sort();

        assert_eq!(
            grammar_keywords, expected,
            "grammar keyword set (alphabetic anonymous node types in node-types.json) drifted \
             from EXPECTED_RESERVED_KEYWORDS. A keyword added to or removed from grammar.js must \
             also be reflected in EXPECTED_RESERVED_KEYWORDS and in src/scanner.c's RESERVED[]."
        );
    }

    /// Direction B: every keyword in `EXPECTED_RESERVED_KEYWORDS` must appear as a
    /// quoted literal inside `src/scanner.c`'s `RESERVED[]` initializer. Catches a
    /// keyword dropped from the scanner (the silent-swallow regression) without
    /// re-running the parser generator.
    #[test]
    fn scanner_reserved_array_matches_expected() {
        const SCANNER_C: &str = include_str!("../../src/scanner.c");
        let start = SCANNER_C
            .find("RESERVED[]")
            .expect("RESERVED[] declaration present in scanner.c");
        let open = SCANNER_C[start..]
            .find('{')
            .map(|i| start + i)
            .expect("RESERVED[] opening brace");
        let close = SCANNER_C[open..]
            .find('}')
            .map(|i| open + i)
            .expect("RESERVED[] closing brace");
        let initializer = &SCANNER_C[open..=close];

        for kw in EXPECTED_RESERVED_KEYWORDS {
            let needle = format!("\"{kw}\"");
            assert!(
                initializer.contains(&needle),
                "src/scanner.c RESERVED[] is missing keyword {needle}; the external scanner will \
                 swallow it into identifiers and constructs using it will parse to ERROR. Keep \
                 RESERVED[] in sync with EXPECTED_RESERVED_KEYWORDS (and grammar.js)."
            );
        }

        // Guard the reverse too: a stray quoted word in RESERVED[] that is not a
        // real grammar keyword would over-reserve and break valid identifiers.
        let quoted: Vec<&str> = initializer
            .split('"')
            .skip(1)
            .step_by(2)
            .filter(|s| !s.is_empty())
            .collect();
        let mut scanner_reserved: Vec<String> = quoted.iter().map(|s| s.to_string()).collect();
        scanner_reserved.sort();
        scanner_reserved.dedup();
        let mut expected: Vec<String> = EXPECTED_RESERVED_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect();
        expected.sort();
        assert_eq!(
            scanner_reserved, expected,
            "src/scanner.c RESERVED[] contents drifted from EXPECTED_RESERVED_KEYWORDS"
        );
    }
}
