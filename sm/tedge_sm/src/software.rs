use std::collections::HashMap;

use mqtt_client::Topic;
use serde::{Deserialize, Serialize};

use crate::message::SoftwareRequestUpdateModule;

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

pub type SoftwareListHash = HashMap<SoftwareType, Vec<SoftwareModule>>;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareListHashStore {
    #[serde(rename = "type")]
    pub module_type: SoftwareListHash,
}

impl SoftwareListHashStore {
    pub fn new(software_list: SoftwareListHash) -> Self {
        Self {
            module_type: software_list,
        }
    }
}

impl Default for SoftwareListHashStore {
    fn default() -> Self {
        Self {
            module_type: HashMap::new(),
        }
    }
}

pub type SoftwareList = Vec<SoftwareModule>;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareListStore {
    // making it pub just not have to implement push for now.
    pub software_list: Vec<SoftwareModule>,
}

impl SoftwareListStore {
    pub fn new(software_list: Vec<SoftwareModule>) -> Self {
        Self { software_list }
    }
}

impl Default for SoftwareListStore {
    fn default() -> Self {
        Self {
            software_list: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum SoftwareOperation {
    // A request for the current software list
    CurrentSoftwareList,

    // A sequence of updates to be applied
    SoftwareUpdates,
}

impl From<Topic> for SoftwareOperation {
    fn from(topic: Topic) -> Self {
        match topic.name.as_str() {
            r#"tedge/commands/req/software/list"# => Self::CurrentSoftwareList,
            r#"tedge/commands/req/software/update"# => Self::SoftwareUpdates,
            _ => todo!(),
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

    // CurrentSoftwareList { list: Vec<SoftwareModule> },
    CurrentSoftwareList { list: SoftwareListHashStore },
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
    #[error("JSON parse error: {reason:?}")]
    ParseError { reason: String },

    #[error("Unknown software type: {software_type:?}")]
    UnknownSoftwareType { software_type: SoftwareType },

    #[error("Unknown {software_type:?} module: {name:?}")]
    UnknownModule {
        software_type: SoftwareType,
        name: SoftwareName,
    },

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

    #[error("Plugin error for {software_type:?}, reason: {reason:?}")]
    Plugin {
        software_type: SoftwareType,
        reason: String,
    },

    #[error("Failed to install {module:?}")]
    Install {
        module: SoftwareRequestUpdateModule,
        reason: String,
    },

    #[error("Failed to prepare")]
    Prepare { reason: String },

    #[error("Failed to finalize")]
    Finalize { reason: String },

    #[error("Failed to uninstall {module:?}")]
    Uninstall {
        module: SoftwareRequestUpdateModule,
        reason: String,
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
