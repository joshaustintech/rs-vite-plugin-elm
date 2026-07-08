pub fn source_dirs_from_elm_json(text: &str) -> Vec<String> {
    let Some(key) = text.find("\"source-directories\"") else {
        return Vec::new();
    };
    let Some(open) = text[key..].find('[').map(|i| key + i) else {
        return Vec::new();
    };
    let Some(close) = text[open..].find(']').map(|i| open + i) else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    let mut chars = text[open + 1..close].chars();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            let mut value = String::new();
            for next in chars.by_ref() {
                if next == '"' {
                    break;
                }
                value.push(next);
            }
            dirs.push(value);
        }
    }
    dirs
}
