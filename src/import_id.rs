use crate::Result;

pub struct ParsedImportId {
    pub path: String,
    pub with: Vec<String>,
    pub valid: bool,
}

pub fn parse(id: &str) -> Result<ParsedImportId> {
    let without_file = id.strip_prefix("file://").unwrap_or(id);
    let (raw_path, query) = without_file
        .split_once('?')
        .map_or((without_file, ""), |(path, query)| (path, query));
    if !without_file.as_bytes().contains(&b'%') {
        let params = query.split('&').filter(|part| !part.is_empty());
        let mut has_raw = false;
        let mut with = Vec::new();

        for param in params {
            let (key, value) = param.split_once('=').unwrap_or((param, ""));
            if key == "raw" {
                has_raw = true;
            } else if key == "with" {
                with.push(value.to_string());
            }
        }

        return Ok(ParsedImportId {
            valid: raw_path.ends_with(".elm") && !has_raw,
            path: raw_path.to_string(),
            with,
        });
    }
    let path = percent_decode(raw_path);
    let params = query.split('&').filter(|part| !part.is_empty());
    let mut has_raw = false;
    let mut with = Vec::new();

    for param in params {
        let (key, value) = param.split_once('=').unwrap_or((param, ""));
        if key == "raw" {
            has_raw = true;
        } else if key == "with" {
            with.push(percent_decode(value));
        }
    }

    Ok(ParsedImportId {
        valid: path.ends_with(".elm") && !has_raw,
        path,
        with,
    })
}

fn percent_decode(input: &str) -> String {
    if !input.as_bytes().contains(&b'%') {
        return input.to_string();
    }
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(hex as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}
