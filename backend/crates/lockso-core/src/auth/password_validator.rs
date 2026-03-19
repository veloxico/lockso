use crate::models::settings::PasswordComplexity;

/// Validation error details.
#[derive(Debug)]
pub struct PasswordValidationError {
    pub violations: Vec<String>,
}

/// Validate a password against complexity requirements.
///
/// Returns Ok(()) if the password meets all requirements,
/// or Err with a list of violations.
pub fn validate_password(
    password: &str,
    rules: &PasswordComplexity,
) -> Result<(), PasswordValidationError> {
    let mut violations = Vec::new();

    if password.len() < rules.min_length as usize {
        violations.push(format!(
            "Password must be at least {} characters",
            rules.min_length
        ));
    }

    if rules.require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        violations.push("Password must contain at least one uppercase letter".to_string());
    }

    if rules.require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        violations.push("Password must contain at least one lowercase letter".to_string());
    }

    if rules.require_digits && !password.chars().any(|c| c.is_ascii_digit()) {
        violations.push("Password must contain at least one digit".to_string());
    }

    if rules.require_special
        && !password
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c.is_ascii())
    {
        violations.push("Password must contain at least one special character".to_string());
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(PasswordValidationError { violations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict_rules() -> PasswordComplexity {
        PasswordComplexity {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
        }
    }

    #[test]
    fn test_valid_password() {
        let result = validate_password("Str0ng!Pass", &strict_rules());
        assert!(result.is_ok());
    }

    #[test]
    fn test_too_short() {
        let result = validate_password("Ab1!", &strict_rules());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .violations
            .iter()
            .any(|v| v.contains("at least 8")));
    }

    #[test]
    fn test_missing_uppercase() {
        let result = validate_password("str0ng!pass", &strict_rules());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_lowercase() {
        let result = validate_password("STR0NG!PASS", &strict_rules());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_digit() {
        let result = validate_password("Strong!Pass", &strict_rules());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_special() {
        let result = validate_password("Str0ngPass1", &strict_rules());
        assert!(result.is_err());
    }

    #[test]
    fn test_relaxed_rules() {
        let rules = PasswordComplexity {
            min_length: 4,
            require_uppercase: false,
            require_lowercase: false,
            require_digits: false,
            require_special: false,
        };
        assert!(validate_password("test", &rules).is_ok());
    }
}
