use argon2::{Argon2, PasswordHash, PasswordVerifier};

pub(super) trait AsApiKey {
    fn prefix(&self, prefix_length: usize) -> &[u8];

    fn is_same_hash(&self, other: &str) -> bool;
}

impl<T> AsApiKey for T
where
    T: AsRef<[u8]>,
{
    fn prefix(&self, prefix_length: usize) -> &[u8] {
        &self.as_ref()[..prefix_length]
    }

    fn is_same_hash(&self, other: &str) -> bool {
        let argon2 = Argon2::default();

        let Ok(parsed_hash) = PasswordHash::new(other) else {
            return false;
        };

        argon2.verify_password(self.as_ref(), &parsed_hash).is_ok()
    }
}
