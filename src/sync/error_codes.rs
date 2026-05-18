use regex::Regex;
use std::sync::LazyLock;

static ERROR_CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\((\d+)\)").unwrap());

pub fn extract(stderr: &str) -> Vec<u32> {
    ERROR_CODE_RE
        .captures_iter(stderr)
        .map(|c| c[1].parse().unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(extract(""), Vec::<u32>::new());
    }

    #[test]
    fn no_parens() {
        assert_eq!(extract("rsync: something went wrong"), Vec::<u32>::new());
    }

    #[test]
    fn single_code() {
        assert_eq!(extract("rsync: write failed (28)"), vec![28]);
    }

    #[test]
    fn multiple_codes_across_lines() {
        let stderr = "rsync: read errors (5)\nrsync: write failed (28)";
        assert_eq!(extract(stderr), vec![5, 28]);
    }

    #[test]
    fn ignores_non_numeric_parens() {
        assert_eq!(extract("rsync: foo (sender) bar (13)"), vec![13]);
    }
}
