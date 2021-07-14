use crate::software::*;
use serde::{Deserialize, Serialize};
use url::Url;

// trait Jsonify {
//     fn from_json(json_str: &str) -> Result<Self, SoftwareError> {
//         Ok(serde_json::from_str(json_str)?)
//     }

//     fn from_slice(bytes: &[u8]) -> Result<Self, SoftwareError> {
//         Ok(serde_json::from_slice(bytes)?)
//     }

//     fn to_json(&self) -> Result<String, SoftwareError> {
//         Ok(serde_json::to_string(self)?)
//     }
// }

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SoftwareRequestList {
    pub id: usize,
}

impl SoftwareRequestList {
    pub fn from_json(json_str: &str) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_json(&self) -> Result<String, SoftwareError> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareRequestUpdate {
    pub id: usize,
    pub update_list: Vec<SoftwareRequestUpdateList>,
}

impl SoftwareRequestUpdate {
    pub fn from_json(json_str: &str) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_json(&self) -> Result<String, SoftwareError> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestUpdateList {
    #[serde(rename = "type")]
    pub plugin_type: SoftwareType,
    pub modules: Vec<SoftwareRequestUpdateModule>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestUpdateModule {
    pub name: SoftwareName,
    pub version: Option<SoftwareVersion>,
    pub action: SoftwareRequestUpdateAction,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestUpdateResponse {
    pub id: usize,
    pub status: SoftwareVersion,

    pub current_software_list: Option<Vec<SoftwareRequestUpdateList>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestUpdateStatus {
    pub update: SoftwareRequestUpdateModule,
    pub status: UpdateStatus,
}

impl SoftwareRequestUpdateStatus {
    pub fn new(update: &SoftwareRequestUpdateModule, result: Result<(), SoftwareError>) -> Self {
        let status = match result {
            Ok(()) => UpdateStatus::Success,
            Err(reason) => UpdateStatus::Error { reason },
        };

        Self {
            update: update.clone(),
            status,
        }
    }

    pub fn scheduled(update: &SoftwareRequestUpdateModule) -> Self {
        Self {
            update: update.clone(),
            status: UpdateStatus::Scheduled,
        }
    }

    pub fn cancelled(update: &SoftwareRequestUpdateModule) -> Self {
        Self {
            update: update.clone(),
            status: UpdateStatus::Cancelled,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareRequestUpdateAction {
    Install,
    Remove,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareOperationResultStatus {
    Successful,
    Failed,
    Executing,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareResponseUpdateStatus {
    pub id: usize,
    pub status: SoftwareOperationResultStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_software_list: Option<Vec<SoftwareRequestUpdateList>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SoftwareResponseUpdateStatus {
    pub fn from_json(json_str: &str) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn to_json(&self) -> Result<String, SoftwareError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, SoftwareError> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SoftwareResponseList {
    pub id: usize,

    pub status: SoftwareOperationResultStatus,

    #[serde(flatten)]
    pub list: SoftwareOperationStatus,
}

impl SoftwareResponseList {
    pub fn from_json(json_str: &str) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn to_json(&self) -> Result<String, SoftwareError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, SoftwareError> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_and_parse_software_updates() {
        let request = SoftwareRequestList {
            id: 42,
            operation: SoftwareOperation::SoftwareUpdates {
                updates: vec![
                    SoftwareUpdate::Install {
                        module: SoftwareModule {
                            software_type: String::from("default"),
                            name: String::from("collectd-core"),
                            version: None,
                            url: None,
                        },
                    },
                    SoftwareUpdate::Install {
                        module: SoftwareModule {
                            software_type: String::from("debian"),
                            name: String::from("ripgrep"),
                            version: None,
                            url: None,
                        },
                    },
                    SoftwareUpdate::UnInstall {
                        module: SoftwareModule {
                            software_type: String::from("default"),
                            name: String::from("hexyl"),
                            version: None,
                            url: None,
                        },
                    },
                ],
            },
        };

        let expected_json = r#"{"id":"42","updates":[{"action":"install","type":"default","name":"collectd-core"},{"action":"install","type":"debian","name":"ripgrep"},{"action":"uninstall","type":"default","name":"hexyl"}]}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request =
            SoftwareRequestList::from_json(&actual_json).expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }

    #[test]
    fn serialize_and_parse_update_status() {
        let status = SoftwareUpdateStatus {
            update: SoftwareUpdate::Install {
                module: SoftwareModule {
                    software_type: String::from("default"),
                    name: String::from("collectd-core"),
                    version: None,
                    url: None,
                },
            },
            status: UpdateStatus::Success,
        };

        let expected_json = r#"{"update":{"action":"install","type":"default","name":"collectd-core"},"status":"Success"}"#;
        let actual_json = serde_json::to_string(&status).expect("Fail to serialize a status");
        assert_eq!(actual_json, expected_json);

        let parsed_status: SoftwareUpdateStatus =
            serde_json::from_str(&actual_json).expect("Fail to parse the json status");
        assert_eq!(parsed_status, status);
    }

    #[test]
    fn serialize_and_parse_software_list() {
        let request = SoftwareRequestList {
            id: String::from("42"),
            operation: SoftwareOperation::CurrentSoftwareList { list: () },
        };
        let expected_json = r#"{"id":"42","list":null}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request =
            SoftwareRequestList::from_json(&actual_json).expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }
}
