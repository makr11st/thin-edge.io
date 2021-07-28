use crate::error::SoftwareError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type SoftwareType = String;
pub type SoftwareName = String;
pub type SoftwareVersion = String;

/// Variants represent Software Operations Supported actions.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareModuleAction {
    Install,
    Remove,
}

/// Software module payload definition.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareModule {
    pub name: SoftwareName,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<SoftwareVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<SoftwareModuleAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum UpdateStatus {
    Scheduled,
    Success,
    Error { reason: SoftwareError },
    Cancelled,
}
