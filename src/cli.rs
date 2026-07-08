use std::path::PathBuf;

pub fn parse_bool(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid bool: {value}").into()),
    }
}

pub fn parse_optional_bool(value: &str) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    if value == "-" {
        Ok(None)
    } else {
        parse_bool(value).map(Some)
    }
}

pub fn parse_optional_string(value: &str) -> Option<String> {
    if value == "-" {
        None
    } else {
        Some(value.to_string())
    }
}

pub fn parse_optional_path(value: &str) -> Option<PathBuf> {
    parse_optional_string(value).map(PathBuf::from)
}

pub fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c => escaped.push(c),
        }
    }
    escaped
}
