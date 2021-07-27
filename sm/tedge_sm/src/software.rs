use crate::error::SoftwareError;
use mqtt_client::Topic;
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

/// Variants of supported software operations.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum SoftwareOperation {
    CurrentSoftwareList,
    SoftwareUpdates,

    #[serde(skip)]
    UnknownOperation,
}

impl From<Topic> for SoftwareOperation {
    fn from(topic: Topic) -> Self {
        match topic.name.as_str() {
            r#"tedge/commands/req/software/list"# => Self::CurrentSoftwareList,
            r#"tedge/commands/req/software/update"# => Self::SoftwareUpdates,
            _ => Self::UnknownOperation,
        }
    }
}

impl From<SoftwareOperation> for Topic {
    fn from(operation: SoftwareOperation) -> Self {
        match operation {
            SoftwareOperation::CurrentSoftwareList => {
                Topic::new("tedge/commands/req/software/list").expect("This is not a topic.")
            }
            SoftwareOperation::SoftwareUpdates => {
                Topic::new("tedge/commands/req/software/update").expect("This is not a topic.")
            }
            SoftwareOperation::UnknownOperation => {
                Topic::new("tedge/commands/unsupported_operation").expect("This is not a topic.")
            }
        }
    }
}

impl FromStr for SoftwareOperation {
    type Err = SoftwareError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            r#"list"# => Ok(Self::CurrentSoftwareList),
            r#"update"# => Ok(Self::SoftwareUpdates),
            _ => Ok(Self::UnknownOperation),
        }
    }
}

impl From<String> for SoftwareOperation {
    fn from(s: String) -> Self {
        match s.as_str() {
            r#"list"# => Self::CurrentSoftwareList,
            r#"update"# => Self::SoftwareUpdates,
            _ => Self::UnknownOperation,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum UpdateStatus {
    Scheduled,
    Success,
    Error { reason: SoftwareError },
    Cancelled,
}
