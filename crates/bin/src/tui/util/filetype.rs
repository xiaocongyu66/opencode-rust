use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

static LANGUAGE_EXTENSIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(".abap", "abap");
    m.insert(".bat", "bat");
    m.insert(".bib", "bibtex");
    m.insert(".bibtex", "bibtex");
    m.insert(".clj", "clojure");
    m.insert(".cljs", "clojure");
    m.insert(".cljc", "clojure");
    m.insert(".edn", "clojure");
    m.insert(".coffee", "coffeescript");
    m.insert(".c", "c");
    m.insert(".cpp", "cpp");
    m.insert(".cxx", "cpp");
    m.insert(".cc", "cpp");
    m.insert(".c++", "cpp");
    m.insert(".cs", "csharp");
    m.insert(".csx", "csharp");
    m.insert(".css", "css");
    m.insert(".d", "d");
    m.insert(".pas", "pascal");
    m.insert(".pascal", "pascal");
    m.insert(".diff", "diff");
    m.insert(".patch", "diff");
    m.insert(".dart", "dart");
    m.insert(".dockerfile", "dockerfile");
    m.insert(".ex", "elixir");
    m.insert(".exs", "elixir");
    m.insert(".erl", "erlang");
    m.insert(".ets", "typescript");
    m.insert(".hrl", "erlang");
    m.insert(".fs", "fsharp");
    m.insert(".fsi", "fsharp");
    m.insert(".fsx", "fsharp");
    m.insert(".fsscript", "fsharp");
    m.insert(".gitcommit", "git-commit");
    m.insert(".gitrebase", "git-rebase");
    m.insert(".go", "go");
    m.insert(".groovy", "groovy");
    m.insert(".gleam", "gleam");
    m.insert(".hbs", "handlebars");
    m.insert(".handlebars", "handlebars");
    m.insert(".hs", "haskell");
    m.insert(".lhs", "haskell");
    m.insert(".html", "html");
    m.insert(".htm", "html");
    m.insert(".ini", "ini");
    m.insert(".java", "java");
    m.insert(".jl", "julia");
    m.insert(".js", "javascript");
    m.insert(".kt", "kotlin");
    m.insert(".kts", "kotlin");
    m.insert(".jsx", "javascriptreact");
    m.insert(".json", "json");
    m.insert(".tex", "latex");
    m.insert(".latex", "latex");
    m.insert(".less", "less");
    m.insert(".lua", "lua");
    m.insert(".makefile", "makefile");
    m.insert("makefile", "makefile");
    m.insert(".md", "markdown");
    m.insert(".markdown", "markdown");
    m.insert(".m", "objective-c");
    m.insert(".mm", "objective-cpp");
    m.insert(".pl", "perl");
    m.insert(".pm", "perl");
    m.insert(".pm6", "perl6");
    m.insert(".php", "php");
    m.insert(".ps1", "powershell");
    m.insert(".psm1", "powershell");
    m.insert(".pug", "jade");
    m.insert(".jade", "jade");
    m.insert(".py", "python");
    m.insert(".r", "r");
    m.insert(".cshtml", "razor");
    m.insert(".razor", "razor");
    m.insert(".rb", "ruby");
    m.insert(".rake", "ruby");
    m.insert(".gemspec", "ruby");
    m.insert(".ru", "ruby");
    m.insert(".erb", "erb");
    m.insert(".html.erb", "erb");
    m.insert(".js.erb", "erb");
    m.insert(".css.erb", "erb");
    m.insert(".json.erb", "erb");
    m.insert(".rs", "rust");
    m.insert(".scss", "scss");
    m.insert(".sass", "sass");
    m.insert(".scala", "scala");
    m.insert(".shader", "shaderlab");
    m.insert(".sh", "shellscript");
    m.insert(".bash", "shellscript");
    m.insert(".zsh", "shellscript");
    m.insert(".ksh", "shellscript");
    m.insert(".sql", "sql");
    m.insert(".svelte", "svelte");
    m.insert(".swift", "swift");
    m.insert(".ts", "typescript");
    m.insert(".tsx", "typescriptreact");
    m.insert(".mts", "typescript");
    m.insert(".cts", "typescript");
    m.insert(".mtsx", "typescriptreact");
    m.insert(".ctsx", "typescriptreact");
    m.insert(".xml", "xml");
    m.insert(".xsl", "xsl");
    m.insert(".yaml", "yaml");
    m.insert(".yml", "yaml");
    m.insert(".mjs", "javascript");
    m.insert(".cjs", "javascript");
    m.insert(".vue", "vue");
    m.insert(".zig", "zig");
    m.insert(".zon", "zig");
    m.insert(".astro", "astro");
    m.insert(".ml", "ocaml");
    m.insert(".mli", "ocaml");
    m.insert(".tf", "terraform");
    m.insert(".tfvars", "terraform-vars");
    m.insert(".hcl", "hcl");
    m.insert(".nix", "nix");
    m.insert(".typ", "typst");
    m.insert(".typc", "typst");
    m
});

pub fn filetype(input: Option<&str>) -> &'static str {
    let input = match input {
        Some(s) if !s.is_empty() => s,
        _ => return "none",
    };

    let ext = Path::new(input)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let ext_with_dot = format!(".{}", ext);
    let language = LANGUAGE_EXTENSIONS.get(ext_with_dot.as_str()).copied();

    match language {
        Some("typescriptreact") | Some("javascriptreact") | Some("javascript") => "typescript",
        Some(lang) => lang,
        None => {
            let basename = Path::new(input)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            LANGUAGE_EXTENSIONS.get(basename).copied().unwrap_or("none")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none() {
        assert_eq!(filetype(None), "none");
        assert_eq!(filetype(Some("")), "none");
    }

    #[test]
    fn test_rust() {
        assert_eq!(filetype(Some("main.rs")), "rust");
    }

    #[test]
    fn test_tsx_to_typescript() {
        assert_eq!(filetype(Some("component.tsx")), "typescript");
    }

    #[test]
    fn test_js_to_typescript() {
        assert_eq!(filetype(Some("index.js")), "typescript");
    }

    #[test]
    fn test_python() {
        assert_eq!(filetype(Some("script.py")), "python");
    }

    #[test]
    fn test_makefile() {
        assert_eq!(filetype(Some("Makefile")), "makefile");
    }

    #[test]
    fn test_unknown() {
        assert_eq!(filetype(Some("file.xyz")), "none");
    }
}
