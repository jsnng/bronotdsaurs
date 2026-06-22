use crate::tds::prelude::*;

#[derive(Debug, Clone)]
pub struct DataClassificationToken {
    pub(crate) sensitivity_labels: Vec<SensitivityLabel>,
    pub(crate) information_types: Vec<InformationType>,
    pub(crate) sensitivity_rank: Vec<SensitivityRank>,
    pub(crate) data_classification_per_column_data: Vec<ColumnSensitivity>,
}

impl<'a> DataClassificationSpan<'a> {
    pub fn ty(&self) -> u8 { self.bytes[0] }
}

#[derive(Debug, Clone)]
pub struct SensitivityLabel {
    name: String,
    label_id: String,
}

#[derive(Debug, Clone)]
pub struct InformationType {
    name: String,
    type_id: String,
}

#[derive(Debug, Clone)]
pub struct ColumnSensitivity {
    properties: Vec<SensitivityProperty>,
}

#[derive(Debug, Clone)]
pub struct SensitivityProperty {
    label_index: u16,
    type_index: u16,
    sensitivity_rank: SensitivityRank,
}
