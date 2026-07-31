use color_eyre::Result;
use color_eyre::eyre::{bail, ensure};

/// Escapes reserved LaTeX characters (`% \ # & _ $ { } ~ ^`) so a value is safe both to tokenize on
/// the pdflatex command line and to typeset verbatim via `\flag`.
fn escape_latex_reserved(input: &str) -> String {
    input.chars().fold(String::new(), |mut s, c| {
        match c {
            '%' | '#' | '&' | '_' | '$' | '{' | '}' => {
                s.push('\\');
                s.push(c);
            }
            '\\' => s.push_str(r"\textbackslash{}"),
            '~' => s.push_str(r"\textasciitilde{}"),
            '^' => s.push_str(r"\textasciicircum{}"),
            _ => s.push(c),
        }
        s
    })
}

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

/// Represents both string and boolean flags passed to the `compile` command.
#[derive(Debug, PartialEq)]
pub enum Flag {
    String { key: String, value: String },
    Boolean { key: String, value: bool },
}

/// Parses a key-value pair in the form "key" or "key=value" into its
/// corresponding flag, validating the key and escaping the value.
pub fn parse_key_val(input: &str) -> Result<Flag> {
    match input.split_once('=') {
        Some((key, value)) => {
            validate_key(key)?;
            Ok(Flag::String {
                key: key.to_string(),
                value: escape_latex_reserved(value),
            })
        }
        None => {
            let (key, value) = match input.strip_prefix("no-") {
                Some(stripped) => (stripped, false),
                None => (input, true),
            };
            validate_key(key)?;
            Ok(Flag::Boolean {
                key: key.to_string(),
                value,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_reserved_characters_escaped() {
        let test_string = "abc[]%\\#&_${}~^def";
        let escaped = escape_latex_reserved(test_string);
        assert_eq!(
            &escaped,
            r"abc[]\%\textbackslash{}\#\&\_\$\{\}\textasciitilde{}\textasciicircum{}def"
        );
    }

    #[rstest]
    #[case("key=value", Flag::String { key: "key".to_string(), value: "value".to_string() })]
    #[case("key=first=second", Flag::String { key: "key".to_string(), value: "first=second".to_string() })]
    #[case("key=&val", Flag::String { key: "key".to_string(), value: r"\&val".to_string() })]
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
