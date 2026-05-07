use macro_attributes::unit_enum;

#[unit_enum]
pub enum Microliter {
    #[cfg_attr(feature = "serde", serde(alias = "µL"))]
    Microliter,
}

#[unit_enum]
pub enum Milliliter {
    #[cfg_attr(feature = "serde", serde(alias = "mL"))]
    Milliliter,
}

#[unit_enum]
pub enum Micrometer {
    #[cfg_attr(feature = "serde", serde(alias = "µm"))]
    Micrometer,
}

#[unit_enum]
pub enum Picogram {
    #[cfg_attr(feature = "serde", serde(alias = "pg"))]
    Picogram,
}

#[unit_enum]
pub enum Nanogram {
    #[cfg_attr(feature = "serde", serde(alias = "ng"))]
    Nanogram,
}
