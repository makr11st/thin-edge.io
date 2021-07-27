use crate::message::SoftwareModulesUpdateRequest;
use mqtt_client::Topic;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type SoftwareType = String;
pub type SoftwareName = String;
pub type SoftwareVersion = String;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SoftwareModule {
    #[serde(skip)]
    pub software_type: SoftwareType,
    pub name: SoftwareName,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<SoftwareVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
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
#[serde(tag = "action")]
pub enum SoftwareUpdate {
    #[serde(rename = "install")]
    Install {
        #[serde(flatten)]
        module: SoftwareModule,
    },

    #[serde(rename = "uninstall")]
    UnInstall {
        #[serde(flatten)]
        module: SoftwareModule,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum SoftwareOperationStatus {
    SoftwareUpdates { updates: Vec<SoftwareUpdateStatus> },
    DesiredSoftwareList { updates: Vec<SoftwareUpdateStatus> },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SoftwareUpdateStatus {
    pub update: SoftwareUpdate,
    pub status: UpdateStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum UpdateStatus {
    Scheduled,
    Success,
    Error { reason: SoftwareError },
    Cancelled,
}

impl SoftwareUpdateStatus {
    pub fn new(update: &SoftwareUpdate, result: Result<(), SoftwareError>) -> SoftwareUpdateStatus {
        let status = match result {
            Ok(()) => UpdateStatus::Success,
            Err(reason) => UpdateStatus::Error { reason },
        };

        SoftwareUpdateStatus {
            update: update.clone(),
            status,
        }
    }

    pub fn scheduled(update: &SoftwareUpdate) -> SoftwareUpdateStatus {
        SoftwareUpdateStatus {
            update: update.clone(),
            status: UpdateStatus::Scheduled,
        }
    }

    pub fn cancelled(update: &SoftwareUpdate) -> SoftwareUpdateStatus {
        SoftwareUpdateStatus {
            update: update.clone(),
            status: UpdateStatus::Cancelled,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SoftwareError {
    #[error("Failed to finalize")]
    Finalize { reason: String },

    #[error("Failed to install {module:?}")]
    Install {
        module: SoftwareModulesUpdateRequest,
        reason: String,
    },

    #[error("JSON parse error: {reason:?}")]
    ParseError { reason: String },

    #[error("Plugin error for {software_type:?}, reason: {reason:?}")]
    Plugin {
        software_type: SoftwareType,
        reason: String,
    },

    #[error("Failed to prepare")]
    Prepare { reason: String },

    #[error("Failed to uninstall {module:?}")]
    Uninstall {
        module: SoftwareModulesUpdateRequest,
        reason: String,
    },

    #[error("Unknown {software_type:?} module: {name:?}")]
    UnknownModule {
        software_type: SoftwareType,
        name: SoftwareName,
    },

    #[error("Unknown software type: {software_type:?}")]
    UnknownSoftwareType { software_type: SoftwareType },

    #[error("Unknown {software_type:?} version: {name:?} - {version:?}")]
    UnknownVersion {
        software_type: SoftwareType,
        name: SoftwareName,
        version: SoftwareVersion,
    },

    #[error("Unexpected module type: actual: {actual_type:?}, expected: {expected_type:?}")]
    WrongModuleType {
        actual_type: SoftwareType,
        expected_type: SoftwareType,
    },
}

impl From<serde_json::Error> for SoftwareError {
    fn from(err: serde_json::Error) -> Self {
        SoftwareError::ParseError {
            reason: format!("{}", err),
        }
    }
}

impl SoftwareUpdate {
    pub fn module(&self) -> &SoftwareModule {
        match self {
            SoftwareUpdate::Install { module } | SoftwareUpdate::UnInstall { module } => module,
        }
    }
}
