//! Tree-sitter parser configuration — WASM URLs and query sources.
//! Ported from tui/src/parsers-config.ts (386 lines)
//!
//! Defines parser configurations for all supported file types, including:
//! - WASM parser URLs from tree-sitter releases
//! - Highlight query URLs from nvim-treesitter
//! - Locals query URLs where available
//! - File type aliases

use std::collections::HashMap;
use std::sync::LazyLock;

/// Query sources for a parser.
#[derive(Debug, Clone)]
pub struct ParserQueries {
    pub highlights: Vec<String>,
    pub locals: Vec<String>,
}

/// Parser configuration for a file type.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub filetype: &'static str,
    pub aliases: Vec<&'static str>,
    pub wasm: String,
    pub queries: ParserQueries,
}

/// All registered parser configurations.
pub static PARSERS: LazyLock<Vec<ParserConfig>> = LazyLock::new(|| {
    vec![
        ParserConfig {
            filetype: "python",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-python/releases/download/v0.23.6/tree-sitter-python.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://github.com/tree-sitter/tree-sitter-python/raw/refs/heads/master/queries/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/python/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "rust",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-rust/releases/download/v0.24.0/tree-sitter-rust.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/rust/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/rust/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "go",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-go/releases/download/v0.25.0/tree-sitter-go.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/go/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/go/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "cpp",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-cpp/releases/download/v0.23.4/tree-sitter-cpp.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/cpp/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/cpp/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "csharp",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-c-sharp/releases/download/v0.23.1/tree-sitter-c_sharp.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/c_sharp/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/c_sharp/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "bash",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-bash/releases/download/v0.25.0/tree-sitter-bash.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/bash/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "c",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-c/releases/download/v0.24.1/tree-sitter-c.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/c/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/c/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "java",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-java/releases/download/v0.23.5/tree-sitter-java.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/java/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/java/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "kotlin",
            aliases: vec![],
            wasm: "https://github.com/fwcd/tree-sitter-kotlin/releases/download/0.3.8/tree-sitter-kotlin.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/fwcd/tree-sitter-kotlin/0.3.8/queries/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/master/queries/kotlin/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "ruby",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-ruby/releases/download/v0.23.1/tree-sitter-ruby.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/ruby/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/ruby/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "php",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-php/releases/download/v0.24.2/tree-sitter-php.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://github.com/tree-sitter/tree-sitter-php/raw/refs/heads/master/queries/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "scala",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-scala/releases/download/v0.24.0/tree-sitter-scala.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/scala/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "html",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-html/releases/download/v0.23.2/tree-sitter-html.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://github.com/tree-sitter/tree-sitter-html/raw/refs/heads/master/queries/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "vue",
            aliases: vec![],
            wasm: "https://github.com/anomalyco/tree-sitter-vue/releases/download/v0.1.2/tree-sitter-vue.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/anomalyco/tree-sitter-vue/v0.1.2/queries/html_tags/highlights.scm".to_string(),
                    "https://raw.githubusercontent.com/anomalyco/tree-sitter-vue/v0.1.2/queries/vue/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "hcl",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-hcl/releases/download/v1.2.0/tree-sitter-hcl.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/master/queries/hcl/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "json",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-json/releases/download/v0.24.8/tree-sitter-json.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/json/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "yaml",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-yaml/releases/download/v0.7.2/tree-sitter-yaml.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/yaml/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "haskell",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-haskell/releases/download/v0.23.1/tree-sitter-haskell.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/haskell/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "css",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-css/releases/download/v0.25.0/tree-sitter-css.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/css/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "julia",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-julia/releases/download/v0.23.1/tree-sitter-julia.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/julia/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "lua",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-lua/releases/download/v0.5.0/tree-sitter-lua.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/tree-sitter-grammars/tree-sitter-lua/v0.5.0/queries/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/tree-sitter-grammars/tree-sitter-lua/v0.5.0/queries/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "ocaml",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-ocaml/releases/download/v0.24.2/tree-sitter-ocaml.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/ocaml/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "clojure",
            aliases: vec![],
            wasm: "https://github.com/anomalyco/tree-sitter-clojure/releases/download/v0.0.1/tree-sitter-clojure.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/clojure/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "swift",
            aliases: vec![],
            wasm: "https://github.com/alex-pinkus/tree-sitter-swift/releases/download/0.7.1/tree-sitter-swift.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/alex-pinkus/tree-sitter-swift/main/queries/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/swift/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "toml",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-toml/releases/download/v0.7.0/tree-sitter-toml.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/master/queries/toml/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "nix",
            aliases: vec![],
            wasm: "https://github.com/ast-grep/ast-grep.github.io/raw/40b84530640aa83a0d34a20a2b0623d7b8e5ea97/website/public/parsers/tree-sitter-nix.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/nix/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/nix/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "diff",
            aliases: vec!["udiff", "patch"],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-diff/releases/download/v0.1.0/tree-sitter-diff.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/tree-sitter-grammars/tree-sitter-diff/master/queries/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "elixir",
            aliases: vec![],
            wasm: "https://github.com/elixir-lang/tree-sitter-elixir/releases/download/v0.3.5/tree-sitter-elixir.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/elixir/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/elixir/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "fsharp",
            aliases: vec![],
            wasm: "https://github.com/ionide/tree-sitter-fsharp/releases/download/0.3.0/tree-sitter-fsharp.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/fsharp/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "r",
            aliases: vec![],
            wasm: "https://github.com/r-lib/tree-sitter-r/releases/download/v1.2.0/tree-sitter-r.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/r/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/r/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "make",
            aliases: vec!["makefile"],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-make/releases/download/v1.1.1/tree-sitter-make.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/make/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
        ParserConfig {
            filetype: "vim",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-vim/releases/download/v0.8.1/tree-sitter-vim.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/vim/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/vim/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "xml",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter-grammars/tree-sitter-xml/releases/download/v0.7.0/tree-sitter-xml.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/xml/highlights.scm".to_string(),
                ],
                locals: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/xml/locals.scm".to_string(),
                ],
            },
        },
        ParserConfig {
            filetype: "agda",
            aliases: vec![],
            wasm: "https://github.com/tree-sitter/tree-sitter-agda/releases/download/v1.3.3/tree-sitter-agda.wasm".to_string(),
            queries: ParserQueries {
                highlights: vec![
                    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/refs/heads/master/queries/agda/highlights.scm".to_string(),
                ],
                locals: vec![],
            },
        },
    ]
});

/// Lookup a parser by filetype (including aliases).
pub fn parser_for_filetype(filetype: &str) -> Option<&'static ParserConfig> {
    PARSERS.iter().find(|p| {
        p.filetype == filetype || p.aliases.contains(&filetype)
    })
}

/// Lookup a parser by file extension.
pub fn parser_for_extension(ext: &str) -> Option<&'static ParserConfig> {
    let ext = ext.trim_start_matches('.');
    PARSERS.iter().find(|p| {
        p.filetype == ext || p.aliases.contains(&ext)
    })
}

/// Build a filetype → parser config map.
pub fn filetype_map() -> HashMap<&'static str, &'static ParserConfig> {
    PARSERS
        .iter()
        .map(|p| (p.filetype, p))
        .collect()
}

/// Count parsers with locals queries.
pub fn count_with_locals() -> usize {
    PARSERS.iter().filter(|p| !p.queries.locals.is_empty()).count()
}

/// Count parsers with aliases.
pub fn count_with_aliases() -> usize {
    PARSERS.iter().filter(|p| !p.aliases.is_empty()).count()
}

/// Get all filetypes.
pub fn all_filetypes() -> Vec<&'static str> {
    PARSERS.iter().map(|p| p.filetype).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_count() {
        assert_eq!(PARSERS.len(), 34);
    }

    #[test]
    fn test_parser_for_filetype_rust() {
        let parser = parser_for_filetype("rust").unwrap();
        assert_eq!(parser.filetype, "rust");
        assert!(!parser.queries.highlights.is_empty());
        assert!(!parser.queries.locals.is_empty());
    }

    #[test]
    fn test_parser_for_filetype_python() {
        let parser = parser_for_filetype("python").unwrap();
        assert_eq!(parser.filetype, "python");
    }

    #[test]
    fn test_parser_for_filetype_alias() {
        let parser = parser_for_filetype("udiff").unwrap();
        assert_eq!(parser.filetype, "diff");
    }

    #[test]
    fn test_parser_for_extension_makefile() {
        let parser = parser_for_extension("makefile").unwrap();
        assert_eq!(parser.filetype, "make");
    }

    #[test]
    fn test_parser_for_filetype_not_found() {
        assert!(parser_for_filetype("nonexistent").is_none());
    }

    #[test]
    fn test_all_filetypes() {
        let types = all_filetypes();
        assert!(types.contains(&"rust"));
        assert!(types.contains(&"python"));
        assert!(types.contains(&"go"));
        assert!(types.contains(&"diff"));
    }

    #[test]
    fn test_filetype_map() {
        let map = filetype_map();
        assert!(map.contains_key("rust"));
        assert!(map.contains_key("python"));
    }

    #[test]
    fn test_count_with_locals() {
        let count = count_with_locals();
        assert!(count > 10);
    }

    #[test]
    fn test_count_with_aliases() {
        let count = count_with_aliases();
        assert!(count >= 2); // diff and make
    }

    #[test]
    fn test_diff_aliases() {
        let parser = parser_for_filetype("patch").unwrap();
        assert_eq!(parser.filetype, "diff");
        assert!(parser.aliases.contains(&"udiff"));
        assert!(parser.aliases.contains(&"patch"));
    }

    #[test]
    fn test_make_aliases() {
        let parser = parser_for_filetype("makefile").unwrap();
        assert_eq!(parser.filetype, "make");
        assert!(parser.aliases.contains(&"makefile"));
    }

    #[test]
    fn test_all_have_wasm_urls() {
        for p in PARSERS.iter() {
            assert!(!p.wasm.is_empty(), "{} has no wasm URL", p.filetype);
            assert!(p.wasm.starts_with("https://"), "{} wasm URL is not https", p.filetype);
        }
    }

    #[test]
    fn test_all_have_highlights() {
        for p in PARSERS.iter() {
            assert!(!p.queries.highlights.is_empty(), "{} has no highlights", p.filetype);
        }
    }
}
