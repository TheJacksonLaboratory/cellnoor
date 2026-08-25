use crate::Error;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "postgres-types",
    derive(postgres_types::FromSql, postgres_types::ToSql)
)]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "String"))]
#[cfg_attr(feature = "postgres-types", postgres(name = "case_insensitive_text"))]
pub struct NonemptyString(String);

impl std::fmt::Display for NonemptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <String as std::fmt::Display>::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for NonemptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <String as std::fmt::Debug>::fmt(&self.0, f)
    }
}

impl NonemptyString {
    pub fn new(s: String) -> Result<Self, Error<String>> {
        if s.is_empty() {
            return Err(Error(s));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for NonemptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<NonemptyString> for String {
    fn from(value: NonemptyString) -> Self {
        value.0
    }
}

impl TryFrom<String> for NonemptyString {
    type Error = Error<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use super::NonemptyString;

    #[test]
    fn deserialize_empty_string_fails() {
        let result: Result<Vec<NonemptyString>, _> = serde_json::from_str(r#"[""]"#);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_non_empty_string_succeeds() {
        let deserialized: [NonemptyString; 1] = serde_json::from_str(r#"["string"]"#).unwrap();

        assert_eq!(
            deserialized,
            [NonemptyString::new("string".to_owned()).unwrap()]
        );
    }
}
