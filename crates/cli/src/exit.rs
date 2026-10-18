use luria_core::error::ErrorCategory;

/// The process exit statuses of plan §3.7.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    /// 0: verified, and every member designed.
    Success,
    /// 1: an internal error or an I/O failure.
    Internal,
    /// 2: bad arguments; text on stderr, no JSON.
    Usage,
    /// 3: `needs_clarification`.
    NeedsClarification,
    /// 4: `failed`, `MalformedInput`.
    MalformedInput,
    /// 5: `failed`, `BiologicalValidation`.
    BiologicalValidation,
    /// 6: `failed`, `Unsupported`.
    Unsupported,
    /// 7: verified, but at least one member, of a set or a single edit, could not be designed.
    PartialDesign,
}

impl ExitStatus {
    /// The process exit status of this state.
    pub fn code(self) -> u8 {
        match self {
            ExitStatus::Success => 0,
            ExitStatus::Internal => 1,
            ExitStatus::Usage => 2,
            ExitStatus::NeedsClarification => 3,
            ExitStatus::MalformedInput => 4,
            ExitStatus::BiologicalValidation => 5,
            ExitStatus::Unsupported => 6,
            ExitStatus::PartialDesign => 7,
        }
    }

    /// The status of a run that failed with an error of this category.
    pub fn for_failure(category: ErrorCategory) -> ExitStatus {
        match category {
            ErrorCategory::MalformedInput => ExitStatus::MalformedInput,
            ErrorCategory::BiologicalValidation => ExitStatus::BiologicalValidation,
            ErrorCategory::Unsupported => ExitStatus::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_its_documented_code() {
        let cases: [(ExitStatus, u8); 8] = [
            (ExitStatus::Success, 0),
            (ExitStatus::Internal, 1),
            (ExitStatus::Usage, 2),
            (ExitStatus::NeedsClarification, 3),
            (ExitStatus::MalformedInput, 4),
            (ExitStatus::BiologicalValidation, 5),
            (ExitStatus::Unsupported, 6),
            (ExitStatus::PartialDesign, 7),
        ];
        for (status, code) in cases {
            assert_eq!(status.code(), code, "{:?}", status);
        }
    }

    #[test]
    fn a_failure_exits_by_its_category() {
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::MalformedInput),
            ExitStatus::MalformedInput
        );
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::BiologicalValidation),
            ExitStatus::BiologicalValidation
        );
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::Unsupported),
            ExitStatus::Unsupported
        );
    }
}
