use crate::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub fn inject(code: &str) -> Result<String> {
    let mut paths = Vec::new();
    let helper_names = helper_asset_functions(code);
    let mut out = String::with_capacity(code.len());
    let mut rest = code;
    let prefix = "'[VITE_PLUGIN_ELM_ASSET:";
    let suffix = "]'";

    while let Some(start) = rest.find(prefix) {
        let abs_start = start + prefix.len();
        out.push_str(&rest[..start]);
        if let Some(end) = rest[abs_start..].find(suffix) {
            let path = &rest[abs_start..abs_start + end];
            paths.push(path.to_string());
            out.push_str(&import_name(path));
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
        paths.extend(helper_paths);
    }

    if paths.is_empty() {
        return Ok(code.to_string());
    }

    let mut seen = HashSet::new();
    let mut imports = String::new();
    for path in paths {
        if seen.insert(path.clone()) {
            imports.push_str("import ");
            imports.push_str(&import_name(&path));
            imports.push_str(" from '");
            imports.push_str(&path);
            imports.push_str("'\n");
        }
    }
    imports.push('\n');
    imports.push_str(&out);
    Ok(imports)
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
