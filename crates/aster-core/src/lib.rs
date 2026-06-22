//! Deterministic matching engine core for Aster.
//!
//! This crate will contain the central limit order book domain model,
//! validation, matching, event log, and replay logic. It is intentionally
//! minimal at the scaffold stage and does not yet implement matching rules.

/// Returns the project name for smoke tests and the placeholder CLI.
pub fn project_name() -> &'static str {
    "Aster"
}

#[cfg(test)]
mod tests {
    use super::project_name;

    #[test]
    fn exposes_project_name() {
        assert_eq!(project_name(), "Aster");
    }
}
