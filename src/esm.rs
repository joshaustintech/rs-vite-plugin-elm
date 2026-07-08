use crate::{Error, Result};

pub fn to_es_module(js: &str) -> Result<String> {
    let export = export_expr(js)?;
    let mut out = js.to_string();
    out = comment_iife_open(&out);
    out = out.replace("'use strict';", "// -- 'use strict';");
    out = out.replace("\"use strict\";", "// -- \"use strict\";");
    out = comment_function(&out, "function _Platform_export");
    out = comment_function(&out, "function _Platform_mergeExports");
    out = comment_final_export(&out);
    out.push_str("\nexport const Elm = ");
    out.push_str(&export);
    out.push_str(";\n");
    Ok(out)
}

fn export_expr(js: &str) -> Result<String> {
    let marker = "_Platform_export(";
    let start = js
        .rfind(marker)
        .ok_or_else(|| Error::new("Could not find Elm export"))?
        + marker.len();
    let end_marker = ");";
    let end = js[start..]
        .find(end_marker)
        .ok_or_else(|| Error::new("Could not find Elm export end"))?
        + start;
    Ok(js[start..end].to_string())
}

fn comment_iife_open(input: &str) -> String {
    let Some(start) = input.find("(function") else {
        return input.to_string();
    };
    let Some(end) = input[start..].find('{').map(|i| start + i + 1) else {
        return input.to_string();
    };
    format!("{}// -- {}{}", &input[..start], &input[start..end], &input[end..])
}

fn comment_function(input: &str, marker: &str) -> String {
    let Some(start) = input.find(marker) else {
        return input.to_string();
    };
    let Some(end) = find_function_end(input, start) else {
        return input.to_string();
    };
    format!("{}/*\n{}\n*/{}", &input[..start], &input[start..end], &input[end..])
}

fn find_function_end(input: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut seen_open = false;
    for (offset, ch) in input[start..].char_indices() {
        match ch {
            '{' => {
                seen_open = true;
                depth += 1;
            }
            '}' if seen_open => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(start + offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn comment_final_export(input: &str) -> String {
    let Some(start) = input.rfind("_Platform_export(") else {
        return input.to_string();
    };
    let end = input[start..]
        .find("}(this));")
        .map(|i| start + i + "}(this));".len())
        .unwrap_or(input.len());
    format!("{}/*\n{}\n*/{}", &input[..start], &input[start..end], &input[end..])
}
