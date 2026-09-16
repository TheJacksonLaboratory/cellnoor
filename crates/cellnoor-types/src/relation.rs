/// A table or view that a type is read from and written to.
///
/// A read selects the relation's whole row as a composite (`select institution
/// from institution`), which is why a filter or an order-by addresses one of
/// its columns as `(institution).name`.
pub trait Relation {
    const NAME: &'static str;
}
