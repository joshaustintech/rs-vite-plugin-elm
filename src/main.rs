mod cli;

use rs_vite_plugin_elm::compile::{compile, dependencies_for_targets, postprocess};
use rs_vite_plugin_elm::options::{EnvOptions, Options};
use rs_vite_plugin_elm::CompileRequest;
use std::env;
use std::io::Read;
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
        "deps" => deps(args.collect()),
        "postprocess" => postprocess_command(args.collect()),
        _ => Err(format!("unknown command: {command}").into()),
    }
}

fn load(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 10 {
        return Err("usage: rs-vite-plugin-elm load <is_build> <debug|-> <optimize|-> <verbose|-> <path_to_elm|-> <cwd|-> <report|-> <docs|-> <process_opts|-> <target> [with...]".into());
    }
    let is_build = cli::parse_bool(&args[0])?;
    let debug = cli::parse_optional_bool(&args[1])?;
    let optimize = cli::parse_optional_bool(&args[2])?;
    let verbose = cli::parse_optional_bool(&args[3])?;
    let path_to_elm = cli::parse_optional_string(&args[4]);
    let cwd = cli::parse_optional_path(&args[5]);
    let report = cli::parse_optional_string(&args[6]);
    let docs = cli::parse_optional_string(&args[7]);
    let process_opts = cli::parse_optional_string(&args[8]);
    let options = Options::from_env(EnvOptions { is_build, debug, optimize, verbose, path_to_elm, cwd, report, docs, process_opts });

    let targets = args[9..].iter().map(PathBuf::from).collect::<Vec<_>>();
    let output = compile(CompileRequest {
        targets,
        options,
        cwd: None,
    })?;
    println!(
        "{{\"code\":\"{}\",\"dependencies\":[{}]}}",
        cli::json_escape(&output.code),
        cli::json_paths(&output.dependencies)
    );
    Ok(())
}

fn deps(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 {
        return Err("usage: rs-vite-plugin-elm deps <cwd|-> <target> [with...]".into());
    }
    let cwd = cli::parse_optional_path(&args[0]);
    let targets = args[1..].iter().map(PathBuf::from).collect::<Vec<_>>();
    let dependencies = dependencies_for_targets(&targets, cwd.as_deref())?;
    println!(
        "[{}]",
        cli::json_paths(&dependencies)
    );
    Ok(())
}

fn postprocess_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        return Err("usage: rs-vite-plugin-elm postprocess <is_build> <current_dir|-> <dependency_count> <dependency...>".into());
    }
    let is_build = cli::parse_bool(&args[0])?;
    let current_dir = cli::parse_optional_path(&args[1]).unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let dependency_count = args[2].parse::<usize>()?;
    if args.len() < 3 + dependency_count {
        return Err("not enough dependencies supplied".into());
    }
    let dependencies = args[3..3 + dependency_count]
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mut code = String::new();
    std::io::stdin().read_to_string(&mut code)?;
    let output = postprocess(&code, &dependencies, is_build, &current_dir)?;
    println!("{{\"code\":\"{}\"}}", cli::json_escape(&output));
    Ok(())
}
