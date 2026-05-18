static INDEX_SET_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^SI-([NA]{2}|[TN]{2}|[GA]{2}|[TS]{2}|[TT]{2})-[A-H]\d{1,2}$").unwrap()
});

static DNA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ACGT]{8}|[ACGT]{10}$").unwrap());

const INDEX_SET_NAME_ERROR_MESSAGE: &str = "malformed index set name";
