use std::sync::LazyLock;

use cellnoor_models::person::{PersonCreation, PersonUpdate};
use regex::Regex;

use crate::validate::Validate;

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error("{email} invalid: {message}")]
pub enum Error {
    Email { email: String, message: String },
}

impl Validate for PersonCreation {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        Ok(validate_email(self.email())?)
    }
}

impl Validate for PersonUpdate {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        if let Some(email) = self.email() {
            validate_email(email)?;
        }

        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), Error> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(Error::Email {
            email: email.to_owned(),
            message: "invalid email".to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::validate::person::EMAIL_REGEX;

    #[rstest]
    fn valid_email() {
        assert!(EMAIL_REGEX.is_match("peter.parker@spiderman.avengers"))
    }

    #[rstest]
    fn email_has_no_domain() {
        assert!(!EMAIL_REGEX.is_match("SpongeBob"))
    }

    #[rstest]
    fn email_contains_space() {
        assert!(!EMAIL_REGEX.is_match("Harry Potter"))
    }
}
