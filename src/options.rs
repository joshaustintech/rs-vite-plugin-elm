#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub is_build: bool,
    pub mode: CompileMode,
    pub verbose: bool,
    pub path_to_elm: String,
    pub report: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileMode {
    Debug,
    Optimize,
    Plain,
}

impl Options {
    pub fn from_env(
        is_build: bool,
        debug: Option<bool>,
        optimize: Option<bool>,
        path_to_elm: Option<String>,
    ) -> Self {
        let debug = debug.unwrap_or(!is_build);
        let optimize = optimize.unwrap_or(!debug && is_build);
        let mode = match (debug, optimize) {
            (true, _) => CompileMode::Debug,
            (false, true) => CompileMode::Optimize,
            (false, false) => CompileMode::Plain,
        };

        Self {
            is_build,
            mode,
            verbose: is_build,
            path_to_elm: path_to_elm.unwrap_or_else(|| "elm".into()),
            report: None,
            docs: None,
        }
    }

    pub fn debug(&self) -> bool {
        matches!(self.mode, CompileMode::Debug)
    }

    pub fn optimize(&self) -> bool {
        matches!(self.mode, CompileMode::Optimize)
    }
}
