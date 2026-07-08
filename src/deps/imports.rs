pub fn imports_from_source(source: &str) -> Vec<String> {
    let mut module_seen = false;
    let mut reading = false;
    let mut imports = Vec::new();
    let mut in_comment = false;

    for line in source.lines() {
        if !module_seen
            && (line.starts_with("module ")
                || line.starts_with("port module ")
                || line.starts_with("effect module "))
        {
            module_seen = true;
            continue;
        }

        if in_comment {
            if line.trim_end().ends_with("-}") {
                in_comment = false;
            }
            continue;
        }

        if module_seen && line.starts_with("import ") {
            reading = true;
        }

        if reading {
            if let Some(rest) = line.strip_prefix("import ") {
                if let Some(name) = rest.split_whitespace().next() {
                    imports.push(name.to_string());
                }
            } else if line.starts_with(' ') || line.trim().is_empty() || line.starts_with("--") {
            } else if line.starts_with("{-") {
                in_comment = true;
            } else {
                break;
            }
        }
    }

    imports
}
