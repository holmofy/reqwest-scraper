pub fn substr_between<'a>(s: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    if let Some(start) = s.find(prefix) {
        let start_pos = start + prefix.len();
        if let Some(end) = s[start_pos..].find(suffix) {
            return Some(&s[start_pos..start_pos + end]);
        }
    }
    None
}
