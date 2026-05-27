use macro_attributes::unit_enum;

#[unit_enum]
pub enum ChromiumDatasetCmdline {
    #[serde(rename = "cellranger-arc count")]
    #[strum(serialize = "cellranger-arc count")]
    CellrangerarcCount,
    #[serde(rename = "cellranger-atac count")]
    #[strum(serialize = "cellranger-atac count")]
    CellrangeratacCount,
    #[serde(rename = "cellranger count")]
    #[strum(serialize = "cellranger count")]
    CellrangerCount,
    #[serde(rename = "cellranger multi")]
    #[strum(serialize = "cellranger multi")]
    CellrangerMulti,
    #[serde(rename = "cellranger vdj")]
    #[strum(serialize = "cellranger vdj")]
    CellrangerVdj,
}
