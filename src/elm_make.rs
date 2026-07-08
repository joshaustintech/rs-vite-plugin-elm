use crate::options::{CompileMode, Options};
use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn args(targets: &[String], output: &str, options: &Options) -> Vec<String> {
    let mut args = vec!["make".to_string()];
    args.extend(targets.iter().cloned());
    args.push("--output".into());
    args.push(output.into());
    match options.mode {
        CompileMode::Debug => args.push("--debug".into()),
        CompileMode::Optimize => args.push("--optimize".into()),
        CompileMode::Plain => {}
    }
    if let Some(report) = &options.report {
        args.push("--report".into());
        args.push(report.clone());
    }
    if let Some(docs) = &options.docs {
        args.push("--docs".into());
        args.push(docs.clone());
    }
    args
}

pub fn compile_to_string(targets: &[String], cwd: &Path, options: &Options) -> Result<String> {
    let output = temp_output_path();
    let output_s = output.to_string_lossy().into_owned();
    let output_result = Command::new(&options.path_to_elm)
        .args(args(targets, &output_s, options))
        .current_dir(cwd)
        .env("LANG", "en_US.UTF-8")
        .output()
        .map_err(|e| Error::new(format!("Could not run Elm compiler \"{}\": {e}", options.path_to_elm)))?;

    if !output_result.status.success() {
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output_result.stdout),
            String::from_utf8_lossy(&output_result.stderr)
        );
        let _ = fs::remove_file(&output);
        return Err(Error::new(format!("Compilation failed\n{combined}")));
    }

    let code = fs::read_to_string(&output)
        .map_err(|e| Error::new(format!("Failed to read Elm compiler output: {e}")))?;
    let _ = fs::remove_file(output);
    Ok(code)
}

fn temp_output_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("rs-vite-plugin-elm-{nanos}.js"))
}
