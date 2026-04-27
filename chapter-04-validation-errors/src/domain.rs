use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskTitle(String);

impl TaskTitle {
    pub fn parse(raw: String) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation("title must not be empty"));
        }
        if trimmed.len() > 120 {
            return Err(DomainError::Validation(
                "title must be at most 120 characters",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("unauthorized")]
    Unauthorized,
    #[error("validation error: {0}")]
    Validation(&'static str),
    #[error("unexpected error")]
    Unexpected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_title_rejects_empty() {
        let err = TaskTitle::parse("   ".to_string()).unwrap_err();
        assert_eq!(err, DomainError::Validation("title must not be empty"));
    }

    #[test]
    fn task_title_rejects_too_long() {
        let err = TaskTitle::parse("a".repeat(121)).unwrap_err();
        assert_eq!(
            err,
            DomainError::Validation("title must be at most 120 characters")
        );
    }

    #[test]
    fn task_title_trims_input() {
        let title = TaskTitle::parse("  ship api  ".to_string()).unwrap();
        assert_eq!(title.as_str(), "ship api");
    }
}
