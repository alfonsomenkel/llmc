use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub contract: Option<String>,
    pub version: Option<u32>,
    pub inputs: Vec<String>,
    pub output_type: OutputType,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Object,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case", deny_unknown_fields)]
pub enum Rule {
    RequiredField { field: String },
    FieldType { field: String, expected: ValueType },
    AllowedValues { field: String, values: Vec<Value> },
    Regex { field: String, pattern: String },
    MinItems { value: u64 },
    NoEmptyRows,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
}
