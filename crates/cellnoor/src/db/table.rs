#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum JunctionTable {
    ProjectAccess,
    SuspensionPreparer,
    SuspensionPoolPreparer,
    CdnaPreparer,
    LibraryPreparer,
    ChromiumDatasetLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Table {
    Institution,
    Person,
    Project,
    Specimen,
    SpecimenMeasurement,
    Suspension,
    SuspensionMeasurement,
    SuspensionPool,

    #[strum(transparent)]
    Junction(JunctionTable),
}
