pub fn get_title(markdown: &str) -> Option<&str> {
    let new_line_idx = markdown.find("\n").unwrap_or(markdown.len());
    let trimmed = markdown[..new_line_idx].trim();

    // The primary header is the note's title
    let beheaded = if trimmed.starts_with('#') {
        let beheaded = &trimmed[1..];
        if !beheaded.starts_with('#') {
            beheaded.trim()
        } else {
            return None;
        }
    } else {
        return None;
    };

    if beheaded.is_empty() {
        return None;
    }

    Some(beheaded)
}

#[cfg(test)]
mod tests {
    use crate::utilities::notes::get_title;

    #[test]
    fn markdown_without_title() {
        assert_eq!(get_title("hello \r\n hey"), None);
        assert_eq!(get_title("## hello \r\n hey"), None);
        assert_eq!(get_title("##hello \r\n hey"), None);
        assert_eq!(get_title(" ## hello \r\n hey"), None);
        assert_eq!(get_title(" ##hello \r\n hey"), None);
    }

    #[test]
    fn markdown_with_title() {
        assert_eq!(get_title("# hello"), Some("hello"));
        assert_eq!(get_title("#hello"), Some("hello"));
        assert_eq!(get_title(" # hello"), Some("hello"));
        assert_eq!(get_title(" #hello"), Some("hello"));
        assert_eq!(get_title("# hello \r\n hey"), Some("hello"));
        assert_eq!(get_title("#hello \r\n hey"), Some("hello"));
        assert_eq!(get_title(" # hello \r\n hey"), Some("hello"));
        assert_eq!(get_title(" #hello \r\n hey"), Some("hello"));
    }
}
