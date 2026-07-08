#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub is_build: bool,
    pub mode: CompileMode,
    pub verbose: bool,
    pub path_to_elm: String,
    pub report: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let mode = CompileMode::from_flags(debug, optimize);

        Self {
            is_build,
            mode,
            verbose: is_build,
            path_to_elm: path_to_elm.unwrap_or_else(|| "elm".into()),
            report: None,
            docs: None,
        }
    }

    pub const fn debug(&self) -> bool {
        self.mode.is_debug()
    }

    pub const fn optimize(&self) -> bool {
        self.mode.is_optimize()
    }
}

impl CompileMode {
    pub const fn from_flags(debug: bool, optimize: bool) -> Self {
        if debug {
            Self::Debug
        } else if optimize {
            Self::Optimize
        } else {
            Self::Plain
        }
    }

    pub const fn is_debug(self) -> bool {
        matches!(self, Self::Debug)
    }

    pub const fn is_optimize(self) -> bool {
        matches!(self, Self::Optimize)
    }
}

#[cfg(test)]
mod tests {
    use super::CompileMode;

    const DEBUG_MODE: CompileMode = CompileMode::from_flags(true, false);
    const OPT_MODE: CompileMode = CompileMode::from_flags(false, true);
    const PLAIN_MODE: CompileMode = CompileMode::from_flags(false, false);

    #[test]
    fn compile_mode_helpers_work_in_const_contexts() {
        assert!(DEBUG_MODE.is_debug());
        assert!(!DEBUG_MODE.is_optimize());
        assert!(OPT_MODE.is_optimize());
        assert!(!OPT_MODE.is_debug());
        assert!(!PLAIN_MODE.is_debug());
        assert!(!PLAIN_MODE.is_optimize());
    }
}
