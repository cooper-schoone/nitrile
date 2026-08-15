use color_eyre::Result;
use color_eyre::eyre::{bail, ensure};

use crate::compilation::flags::Flag;

/// Validates a flag to ensure that it only contains ASCII letters, digits, and hyphens.
fn validate_key(key: &str) -> Result<()> {
    ensure!(!key.is_empty(), "flag name must not be empty");
    if let Some(bad) = key
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '-'))
    {
        bail!(
            "invalid character {bad:?} in flag name `{key}`; \
             names may contain only ASCII letters, digits, and hyphens"
        );
    }
    Ok(())
}

/// Parses a key-value pair in the form "key" or "key=value" into its
/// corresponding flag and validates the key.
pub fn parse_key_val(input: &str) -> Result<Flag> {
    if let Some((key, value)) = input.split_once('=') {
        validate_key(key)?;
        Ok(Flag::String {
            key: key.to_string(),
            value: value.to_string(),
        })
    } else {
        let (key, value) = input
            .strip_prefix("no-")
            .map_or((input, true), |stripped| (stripped, false));
        validate_key(key)?;
        Ok(Flag::Boolean {
            key: key.to_string(),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("key=value", Flag::String { key: "key".to_string(), value: "value".to_string() })]
    #[case("key=first=second", Flag::String { key: "key".to_string(), value: "first=second".to_string() })]
    #[case("key=&val", Flag::String { key: "key".to_string(), value: "&val".to_string() })]
    #[case("show-summary=on", Flag::String { key: "show-summary".to_string(), value: "on".to_string() })]
    #[case("arr=a[0]", Flag::String { key: "arr".to_string(), value: "a[0]".to_string() })]
    #[case("key", Flag::Boolean { key: "key".to_string(), value: true })]
    #[case("no-key", Flag::Boolean { key: "key".to_string(), value: false })]
    #[case("nokey", Flag::Boolean { key: "nokey".to_string(), value: true })]
    fn test_key_vals_parse_correctly(#[case] input: &str, #[case] expected: Flag) {
        let flag = parse_key_val(input).unwrap();
        assert_eq!(flag, expected);
    }

    #[rstest]
    #[case("key$=^val[]")]
    #[case("$key")]
    #[case("=val")]
    #[case("")]
    #[case("no-$x")]
    fn test_invalid_keys_are_rejected(#[case] input: &str) {
        assert!(parse_key_val(input).is_err());
    }
}
