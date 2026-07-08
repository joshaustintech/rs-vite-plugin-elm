use crate::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub fn inject(code: &str) -> Result<String> {
    if !code.contains("[VITE_PLUGIN_ELM_ASSET:") && !code.contains("VITE_PLUGIN_HELPER_ASSET") {
        return Ok(code.to_string());
    }
    let mut plain_paths = Vec::new();
    let mut plain_seen = HashSet::new();
    let mut plain_names = HashMap::new();
    let mut helper_import_paths = Vec::new();
    let mut helper_seen = HashSet::new();
    let mut helper_name_cache = HashMap::new();
    let helper_names = if code.contains("VITE_PLUGIN_HELPER_ASSET") {
        helper_asset_functions(code)
    } else {
        Vec::new()
    };
    let mut out = String::with_capacity(code.len());
    let mut rest = code;
    let prefix = "'[VITE_PLUGIN_ELM_ASSET:";
    let suffix = "]'";

    while let Some(start) = rest.find(prefix) {
        let abs_start = start + prefix.len();
        out.push_str(&rest[..start]);
        if let Some(end) = rest[abs_start..].find(suffix) {
            let path = &rest[abs_start..abs_start + end];
            record_plain_path(path, &mut plain_paths, &mut plain_seen);
            let name = {
                let entry = plain_names.entry(path).or_insert_with(|| import_name(path));
                entry.clone()
            };
            out.push_str(&name);
            rest = &rest[abs_start + end + suffix.len()..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
        }
    }
    out.push_str(rest);

    for helper in helper_names {
        let (next, helper_paths) = replace_helper_calls(&out, &helper);
        out = next;
        for path in helper_paths {
            record_owned_path(path, &mut helper_import_paths, &mut helper_seen);
        }
    }

    if plain_paths.is_empty() && helper_import_paths.is_empty() {
        return Ok(code.to_string());
    }

    let mut imports = String::new();
    for path in plain_paths {
        imports.push_str("import ");
        let name = {
            let entry = plain_names.entry(path).or_insert_with(|| import_name(path));
            entry.clone()
        };
        imports.push_str(&name);
        imports.push_str(" from '");
        imports.push_str(path);
        imports.push_str("'\n");
    }
    for path in helper_import_paths {
        imports.push_str("import ");
        let name = {
            let entry = helper_name_cache
                .entry(path.clone())
                .or_insert_with(|| import_name(&path));
            entry.clone()
        };
        imports.push_str(&name);
        imports.push_str(" from '");
        imports.push_str(&path);
        imports.push_str("'\n");
    }
    imports.push('\n');
    imports.push_str(&out);
    Ok(imports)
}

fn record_plain_path<'a>(path: &'a str, import_paths: &mut Vec<&'a str>, seen_paths: &mut HashSet<&'a str>) {
    if seen_paths.insert(path) {
        import_paths.push(path);
    }
}

fn record_owned_path(path: String, import_paths: &mut Vec<String>, seen_paths: &mut HashSet<String>) {
    if seen_paths.insert(path.clone()) {
        import_paths.push(path);
    }
}

fn import_name(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("_asset_{:x}", hasher.finish())
}

fn helper_asset_functions(code: &str) -> Vec<String> {
    let marker = "return 'VITE_PLUGIN_HELPER_ASSET' + path;";
    let mut names = Vec::new();
    let mut rest = code;
    while let Some(marker_at) = rest.find(marker) {
        let before = &rest[..marker_at];
        if let Some(var_at) = before.rfind("var ") {
            let name_start = var_at + 4;
            if let Some(eq_at) = before[name_start..].find(" = function") {
                names.push(before[name_start..name_start + eq_at].trim().to_string());
            }
        }
        rest = &rest[marker_at + marker.len()..];
    }
    names
}

fn replace_helper_calls(code: &str, helper: &str) -> (String, Vec<String>) {
    let mut out = String::with_capacity(code.len());
    let mut paths = Vec::new();
    let mut rest = code;
    let call = format!("{helper}('");

    while let Some(start) = rest.find(&call) {
        out.push_str(&rest[..start]);
        let path_start = start + call.len();
        if let Some(end) = rest[path_start..].find("')") {
            let path = &rest[path_start..path_start + end];
            paths.push(path.to_string());
            out.push_str(&import_name(path));
            rest = &rest[path_start + end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
        }
    }
    out.push_str(rest);
    (out, paths)
}
