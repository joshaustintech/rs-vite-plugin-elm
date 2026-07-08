pub mod assets;
pub mod deps;
pub mod elm_make;
pub mod esm;
pub mod hmr;
pub mod import_id;
pub mod options;

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct CompileRequest {
    pub targets: Vec<PathBuf>,
    pub options: options::Options,
    pub cwd: Option<PathBuf>,
}

pub struct CompileOutput {
    pub code: String,
    pub dependencies: Vec<PathBuf>,
}

pub fn compile(request: CompileRequest) -> Result<CompileOutput> {
    let first = request
        .targets
        .first()
        .ok_or_else(|| Error::new("No Elm targets supplied"))?;
    let cwd = request
        .cwd
        .clone()
        .or_else(|| deps::find_elm_json(first).and_then(|p| p.parent().map(Path::to_path_buf)))
        .ok_or_else(|| Error::new("Could not find elm.json"))?;

    let mut dependencies = Vec::new();
    for target in &request.targets {
        dependencies.extend(deps::dependencies(target)?);
    }
    dependencies.sort();
    dependencies.dedup();

    let target_strings = request
        .targets
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let compiled = elm_make::compile_to_string(&target_strings, &cwd, &request.options)?;
    let esm = esm::to_es_module(&compiled)?;
    let with_assets = assets::inject(&esm)?;
    let code = if request.options.is_build {
        with_assets
    } else {
        let trimmed = with_assets.replace(
            "console.warn('Compiled in DEBUG mode",
            "// console.warn('Compiled in DEBUG mode",
        );
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let deps = dependencies
            .iter()
            .map(|path| vite_project_path(path, &current_dir))
            .collect::<Vec<_>>();
        hmr::inject(&trimmed, &deps)?
    };

    Ok(CompileOutput { code, dependencies })
}

fn vite_project_path(path: &Path, current_dir: &Path) -> String {
    let relative = lexical_relative(current_dir, path).unwrap_or_else(|| path.to_string_lossy().into_owned());
    format!("/{}", relative)
}

fn lexical_relative(from: &Path, to: &Path) -> Option<String> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    if from.first() != to.first() {
        return None;
    }
    let mut common = 0;
    while common < from.len() && common < to.len() && from[common] == to[common] {
        common += 1;
    }
    let mut parts = Vec::new();
    for _ in common..from.len() {
        parts.push("..".to_string());
    }
    for component in &to[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    if parts.is_empty() {
        Some(".".into())
    } else {
        Some(parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_elm_import_ids() -> Result<()> {
        let parsed = import_id::parse("/src/Main.elm?with=./Other.elm&with=./Third.elm")?;
        assert_eq!(parsed.path, "/src/Main.elm");
        assert_eq!(parsed.with, vec!["./Other.elm", "./Third.elm"]);
        assert!(parsed.valid);
        assert!(!import_id::parse("/src/Main.elm?raw")?.valid);
        Ok(())
    }

    #[test]
    fn parses_options_without_invalid_debug_optimize_state() {
        let dev = options::Options::from_env(false, None, None, None);
        assert!(dev.debug());
        assert!(!dev.optimize());

        let prod = options::Options::from_env(true, None, None, None);
        assert!(!prod.debug());
        assert!(prod.optimize());

        let debug_prod = options::Options::from_env(true, Some(true), None, None);
        assert!(debug_prod.debug());
        assert!(!debug_prod.optimize());
    }

    #[test]
    fn converts_elm_iife_to_esm() -> Result<()> {
        let js = "(function (scope) {\n'use strict';\nfunction _Platform_export(exports)\n{\nscope['Elm'] = _Platform_mergeExports(scope['Elm'], exports);\n}\nfunction _Platform_mergeExports(module, exports)\n{\nreturn exports;\n}\n_Platform_export({'Hello':{'init':main}});\n}(this));";
        let out = esm::to_es_module(js)?;
        assert!(out.contains("export const Elm = {'Hello':{'init':main}};"));
        assert!(out.contains("// -- (function (scope) {"));
        assert!(out.contains("/*\nfunction _Platform_export"));
        Ok(())
    }

    #[test]
    fn injects_asset_imports() -> Result<()> {
        let out = assets::inject("const x = '[VITE_PLUGIN_ELM_ASSET:/assets/logo.png]';")?;
        assert!(out.contains("import _asset_"));
        assert!(out.contains("from '/assets/logo.png'"));
        assert!(!out.contains("VITE_PLUGIN_ELM_ASSET"));
        Ok(())
    }

    #[test]
    fn injects_helper_asset_calls() -> Result<()> {
        let code = "var $helper$asset = function (path) {\n\treturn 'VITE_PLUGIN_HELPER_ASSET' + path;\n};\nconst x = $helper$asset('/assets/logo.png?inline');";
        let out = assets::inject(code)?;
        assert!(out.contains("import _asset_"));
        assert!(out.contains("from '/assets/logo.png?inline'"));
        assert!(!out.contains("$helper$asset('/assets/logo.png?inline')"));
        Ok(())
    }

    #[test]
    fn builds_elm_make_args() {
        let opts = options::Options {
            is_build: true,
            mode: options::CompileMode::Optimize,
            verbose: true,
            path_to_elm: "elm".into(),
            report: Some("json".into()),
            docs: None,
        };
        let args = elm_make::args(&["Main.elm".into()], "/tmp/out.js", &opts);
        assert_eq!(args, vec!["make", "Main.elm", "--output", "/tmp/out.js", "--optimize", "--report", "json"]);
    }

    #[test]
    fn scans_imports() {
        let src = "module Main exposing (main)\n\nimport Html\nimport Foo.Bar exposing (x)\n\nmain = Html.text \"x\"\n";
        assert_eq!(deps::imports_from_source(src), vec!["Html", "Foo.Bar"]);
    }

    #[test]
    fn injects_hmr_only_for_dev() -> Result<()> {
        let code = "function() { key.a(onUrlChange(_Browser_getUrl())); };\nvar elm$browser$Browser$application = 1;";
        let out = hmr::inject(code, &["/src/Main.elm".into()])?;
        assert!(out.contains("key['elm-hot-nav-key'] = true;"));
        assert!(out.contains("import.meta.hot"));
        Ok(())
    }
}
