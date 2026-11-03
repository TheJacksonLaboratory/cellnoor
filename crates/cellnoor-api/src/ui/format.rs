use cellnoor_types::{
    cdna::creation::LibraryType,
    permission::{Action, Resource},
    person::Person,
    specimen::{
        Fixative, Species, SpecimenType, ThermalPreservationMethod,
        creation::{
            ControlledRateFreezing, DithiobisSuccinimidylpropionate, FlashFreezing,
            FormaldehydeDerivative, block::BlockEmbeddingMatrix,
        },
    },
    tenx_assay::{SampleMultiplexing, TenxAssay},
};
use strum::VariantArray;

use crate::ui::{UiError, params::Opt};

pub(super) fn format_enum(value: &str) -> String {
    value.replace('_', " ")
}

pub(super) fn assay_label(assay: &TenxAssay) -> String {
    match assay.sample_multiplexing {
        Some(multiplexing) => format!("{} ({})", assay.name, multiplexing.label()),
        None => assay.name.to_string(),
    }
}

pub(super) trait Choice: Copy + 'static {
    const ALL: &'static [Self];

    fn value(&self) -> &str;

    fn label(&self) -> String {
        format_enum(self.value())
    }
}

pub(super) fn options<T: Choice>() -> Vec<Opt> {
    T::ALL
        .iter()
        .map(|choice| Opt::new(choice.value(), choice.label()))
        .collect()
}

pub(super) fn parse_choices<T: Choice>(values: &[&str]) -> Result<Vec<T>, UiError> {
    values
        .iter()
        .map(|&value| {
            T::ALL
                .iter()
                .copied()
                .find(|choice| choice.value() == value)
                .ok_or_else(|| UiError::BadRequest(format!("invalid value `{value}`")))
        })
        .collect()
}

pub(super) fn staff_options() -> Vec<Opt> {
    vec![Opt::new("true", "Staff"), Opt::new("false", "Not staff")]
}

pub(super) fn person_options(people: &[Person]) -> Vec<Opt> {
    people
        .iter()
        .map(|person| Opt::new(person.record.id.to_string(), person.record.name.to_string()))
        .collect()
}

impl Choice for Species {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }

    fn label(&self) -> String {
        let label = format_enum(self.value());
        let mut chars = label.chars();

        chars
            .next()
            .map(|first| first.to_uppercase().chain(chars).collect())
            .unwrap_or_default()
    }
}

impl Choice for SpecimenType {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }
}

impl Choice for BlockEmbeddingMatrix {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }

    fn label(&self) -> String {
        match self {
            Self::OptimalCuttingTemperatureCompound => "OCT".to_owned(),
            Self::CarboxymethylCellulose => "CMC".to_owned(),
            Self::Paraffin => format_enum(self.value()),
        }
    }
}

impl Choice for Fixative {
    const ALL: &'static [Self] = &[
        Self::DithiobisSuccinimidylpropionate(
            DithiobisSuccinimidylpropionate::DithiobisSuccinimidylpropionate,
        ),
        Self::FormaldehydeDerivative(FormaldehydeDerivative::FormaldehydeDerivative),
    ];

    fn value(&self) -> &str {
        match self {
            Self::DithiobisSuccinimidylpropionate(fixative) => fixative.as_ref(),
            Self::FormaldehydeDerivative(fixative) => fixative.as_ref(),
        }
    }

    fn label(&self) -> String {
        match self {
            Self::DithiobisSuccinimidylpropionate(_) => "DSP".to_owned(),
            Self::FormaldehydeDerivative(_) => format_enum(self.value()),
        }
    }
}

impl Choice for ThermalPreservationMethod {
    const ALL: &'static [Self] = &[
        Self::Crf(ControlledRateFreezing::ControlledRateFreezing),
        Self::Ff(FlashFreezing::FlashFreezing),
    ];

    fn value(&self) -> &str {
        match self {
            Self::Crf(method) => method.as_ref(),
            Self::Ff(method) => method.as_ref(),
        }
    }
}

impl Choice for SampleMultiplexing {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }

    fn label(&self) -> String {
        match self {
            Self::OnChipMultiplexing => "OCM".to_owned(),
            _ => format_enum(self.value()),
        }
    }
}

impl Choice for LibraryType {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }
}

impl Choice for Action {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }
}

impl Choice for Resource {
    const ALL: &'static [Self] = Self::VARIANTS;

    fn value(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        specimen::{Fixative, Species, creation::block::BlockEmbeddingMatrix},
        tenx_assay::SampleMultiplexing,
    };
    use pretty_assertions::assert_eq;

    use super::{Choice, parse_choices};

    #[test]
    fn labels_match_the_svelte_ui() {
        assert_eq!(Species::HomoSapiens.label(), "Homo sapiens");
        assert_eq!(
            BlockEmbeddingMatrix::OptimalCuttingTemperatureCompound.label(),
            "OCT"
        );
        assert_eq!(Fixative::ALL[0].label(), "DSP");
        assert_eq!(Fixative::ALL[1].label(), "formaldehyde derivative");
        assert_eq!(SampleMultiplexing::OnChipMultiplexing.label(), "OCM");
        assert_eq!(SampleMultiplexing::FlexBarcode.label(), "flex barcode");
    }

    #[test]
    fn choices_parse_from_their_values() {
        let parsed: Vec<Fixative> = parse_choices(&["formaldehyde_derivative"]).unwrap();

        assert_eq!(parsed, [Fixative::ALL[1]]);
        parse_choices::<Species>(&["homo sapiens"]).unwrap_err();
    }
}
