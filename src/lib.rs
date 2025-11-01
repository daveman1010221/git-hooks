use std::fmt;

const TYPES: &[&str] = &[
    "build","chore","ci","docs","feat","fix","perf",
    "refactor","revert","style","test",
];

#[derive(Debug, PartialEq, Eq)]
pub enum CommitError {
    Empty,
    InvalidFormat,
    Io,
}

impl fmt::Display for CommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitError::Empty => write!(f, "empty commit message"),
            CommitError::InvalidFormat => write!(f, "invalid conventional commits header"),
            CommitError::Io => write!(f, "io error"),
        }
    }
}

/// Core logic:
/// - normalize newlines
/// - drop `#` comment lines
/// - trim leading/trailing blanks
/// - lowercase accidental leading Type:
/// - enforce blank line after subject
/// - always end with \n
pub fn normalize_commit_message(raw: &str) -> Result<String, CommitError> {
    // Normalize newlines and strip comments
    let mut lines: Vec<String> = raw
        .replace("\r\n", "\n")
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .map(|l| l.trim_end().to_string())
        .collect();

    // Remove leading/trailing blank lines
    while matches!(lines.first(), Some(l) if l.trim().is_empty()) {
        lines.remove(0);
    }
    while matches!(lines.last(), Some(l) if l.trim().is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        return Err(CommitError::Empty);
    }

    let first_line = lines[0].trim();

    // Maybe downcase accidental `Feat:` → `feat:`
    let mut normalized_first = first_line.to_string();
    for t in TYPES {
        if normalized_first.starts_with(&format!("{}:", capitalize(t)))
            || normalized_first.starts_with(&format!("{}(", capitalize(t)))
        {
            // replace just the prefix
            normalized_first.replace_range(..t.len(), t);
            lines[0] = normalized_first.clone();
            break;
        }
    }

    // Validate
    let valid = TYPES.iter().any(|t| {
        normalized_first.starts_with(&format!("{t}:"))
            || normalized_first.starts_with(&format!("{t}("))
    });

    if !valid {
        return Err(CommitError::InvalidFormat);
    }

    // Ensure exactly one blank line after subject
    if lines.len() > 1 && !lines[1].is_empty() {
        lines.insert(1, String::new());
    }

    // Always end with newline
    let cleaned = lines.join("\n") + "\n";
    Ok(cleaned)
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        let r = normalize_commit_message("");
        assert_eq!(r, Err(CommitError::Empty));
    }

    #[test]
    fn accepts_minimal_valid() {
        let r = normalize_commit_message("feat: ok");
        assert_eq!(r.unwrap(), "feat: ok\n");
    }

    #[test]
    fn allows_scope_form() {
        let r = normalize_commit_message("fix(auth): bug");
        assert_eq!(r.unwrap(), "fix(auth): bug\n");
    }

    #[test]
    fn lowercases_accidental_capital() {
        let r = normalize_commit_message("Feat: title");
        assert_eq!(r.unwrap(), "feat: title\n");
    }

    #[test]
    fn strips_comments_and_blanks() {
        let raw = r#"
# this is a comment
feat: header
body line
# another comment
"#;
        let out = normalize_commit_message(raw).unwrap();
        assert_eq!(out, "feat: header\n\nbody line\n");
    }

    #[test]
    fn enforces_single_blank_line_after_subject() {
        let raw = "feat: header\nstuff\n";
        let out = normalize_commit_message(raw).unwrap();
        assert_eq!(out, "feat: header\n\nstuff\n");
    }

    // ── property-ish tests ────────────────────────────────────────────────

    // 1) running it twice should be idempotent
    #[test]
    fn idempotent_clean() {
        let raw = "feat: header\n\nbody\n";
        let once = normalize_commit_message(raw).unwrap();
        let twice = normalize_commit_message(&once).unwrap();
        assert_eq!(once, twice);
    }

    // 2) comments-only → error
    #[test]
    fn comments_only_is_empty() {
        let raw = "# hi\n# there\n  # indented\n";
        let res = normalize_commit_message(raw);
        assert_eq!(res, Err(CommitError::Empty));
    }

    // 3) windows newlines normalized
    #[test]
    fn windows_newlines_normalized() {
        let raw = "feat: header\r\n\r\nbody\r\n";
        let out = normalize_commit_message(raw).unwrap();
        assert_eq!(out, "feat: header\n\nbody\n");
    }
}

#[cfg(test)]
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // any string → running normalize twice is stable
        #[test]
        fn normalize_is_idempotent(s in ".*") {
            // we only care about ones that pass the first time
            if let Ok(cleaned) = normalize_commit_message(&s) {
                let cleaned2 = normalize_commit_message(&cleaned).unwrap();
                prop_assert_eq!(cleaned, cleaned2);
            }
        }

        // we never produce a message without trailing \n
        #[test]
        fn output_always_has_trailing_newline(s in ".*") {
            if let Ok(cleaned) = normalize_commit_message(&s) {
                prop_assert!(cleaned.ends_with('\n'));
            }
        }
    }
}
