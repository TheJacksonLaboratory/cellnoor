/// A table or view that a type is read from and written to.
pub trait Relation {
    const NAME: &'static str;
}
