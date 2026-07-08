#![forbid(unsafe_code)]

use rs_vite_plugin_elm::options::Options;
use rs_vite_plugin_elm::{compile, CompileRequest};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err("missing command".into());
    };
    match command.as_str() {
        "load" => load(args.collect()),
        _ => Err(format!("unknown command: {command}").into()),
    }
}

fn load(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 6 {
        return Err("usage: rs-vite-plugin-elm load <is_build> <debug|-> <optimize|-> <verbose> <path_to_elm|-> <target> [with...]".into());
    }
    let is_build = parse_bool(&args[0])?;
    let debug = parse_optional_bool(&args[1])?;
    let optimize = parse_optional_bool(&args[2])?;
    let _verbose = parse_bool(&args[3])?;
    let path_to_elm = if args[4] == "-" {
        None
    } else {
        Some(args[4].clone())
    };
    let mut options = Options::from_env(is_build, debug, optimize, path_to_elm);
    options.verbose = parse_bool(&args[3])?;

    let targets = args[5..].iter().map(PathBuf::from).collect::<Vec<_>>();
    let output = compile(CompileRequest {
        targets,
        options,
        cwd: None,
    })?;
    println!(
        "{{\"code\":\"{}\",\"dependencies\":[{}]}}",
        json_escape(&output.code),
        output
            .dependencies
            .iter()
            .map(|path| format!("\"{}\"", json_escape(&path.to_string_lossy())))
            .collect::<Vec<_>>()
            .join(",")
    );
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid bool: {value}").into()),
    }
}

fn parse_optional_bool(value: &str) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    if value == "-" {
        Ok(None)
    } else {
        parse_bool(value).map(Some)
    }
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            c => vec![c],
        })
        .collect()
}
