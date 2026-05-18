#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    Success,
    PartialSuccess { skipped_count_hint: u32 },
    Failure(FailureKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    DriveDisconnected,
    PermissionDenied,
    SourceMissing,
    OutOfSpace,
    Timeout,
    Other { exit_code: i32 },
}

use super::error_codes;

pub fn parse(exit_code: i32, _stdout: &str, stderr: &str) -> RunResult {
    if exit_code == 0 {
        return RunResult::Success;
    }

    let error_codes = error_codes::extract(stderr);

    if error_codes.contains(&28) {
        return RunResult::Failure(FailureKind::OutOfSpace);
    }
    if error_codes.contains(&13) {
        return RunResult::Failure(FailureKind::PermissionDenied);
    }
    if error_codes.contains(&5) || error_codes.contains(&16) || error_codes.contains(&107) {
        return RunResult::Failure(FailureKind::DriveDisconnected);
    }
    if error_codes.contains(&2) {
        return RunResult::Failure(FailureKind::SourceMissing);
    }

    match exit_code {
        23 | 24 => {
            let skipped_count_hint = stderr.lines().filter(|l| !l.is_empty()).count() as u32;
            RunResult::PartialSuccess { skipped_count_hint }
        }
        30 => RunResult::Failure(FailureKind::Timeout),
        11 | 12 => RunResult::Failure(FailureKind::DriveDisconnected),
        code => RunResult::Failure(FailureKind::Other { exit_code: code }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_zero_is_success() {
        assert_eq!(parse(0, "", ""), RunResult::Success);
    }

    #[test]
    fn out_of_space_detected_by_errno_28() {
        assert_eq!(parse(11, "", "rsync: anything (28)"), RunResult::Failure(FailureKind::OutOfSpace));
    }

    #[test]
    fn drive_disconnected_via_errno_5() {
        assert_eq!(
            parse(23, "", "rsync: anything (5)"),
            RunResult::Failure(FailureKind::DriveDisconnected)
        );
    }

    #[test]
    fn drive_disconnected_via_exit_11_no_errno() {
        assert_eq!(
            parse(11, "", ""),
            RunResult::Failure(FailureKind::DriveDisconnected)
        );
    }

    #[test]
    fn permission_denied_via_errno_13() {
        assert_eq!(
            parse(23, "", "rsync: anything (13)"),
            RunResult::Failure(FailureKind::PermissionDenied)
        );
    }

    #[test]
    fn source_missing_via_errno_2() {
        assert_eq!(parse(23, "", "rsync: anything (2)"), RunResult::Failure(FailureKind::SourceMissing));
    }

    #[test]
    fn partial_success_exit_23_without_known_pattern() {
        let stderr = "file vanished: \"a\"\nfile vanished: \"b\"";
        match parse(23, "", stderr) {
            RunResult::PartialSuccess { skipped_count_hint } => {
                assert_eq!(skipped_count_hint, 2);
            }
            other => panic!("expected PartialSuccess, got {other:?}"),
        }
    }

    #[test]
    fn timeout_exit_30() {
        assert_eq!(parse(30, "", ""), RunResult::Failure(FailureKind::Timeout));
    }

    #[test]
    fn unknown_exit_code_falls_through_to_other() {
        assert_eq!(
            parse(99, "", ""),
            RunResult::Failure(FailureKind::Other { exit_code: 99 })
        );
    }

    #[test]
    fn out_of_space_takes_priority_over_exit_code_classification() {
        assert_eq!(parse(23, "", "(28)"), RunResult::Failure(FailureKind::OutOfSpace));
    }
}
