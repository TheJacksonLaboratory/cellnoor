use rand::{RngExt, distr::Alphanumeric};

fn generate_secret() -> String {
    // This gets 128 bits of entropy
    const SECRET_LEN: usize = 22;

    let rng = rand::rng();

    rng.sample_iter(Alphanumeric)
        .take(22)
        .map(char::from)
        .collect()
}
