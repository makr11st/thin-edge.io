use crate::software::*;
use serde::{Deserialize, Serialize};

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

/// Message payload definition for SoftwareList request.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
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

/// Message payload definition for SoftwareUpdate request.
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
    pub list: Vec<SoftwareModulesUpdateRequest>,
}

/// Software module payload definition.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareModulesUpdateRequest {
    pub name: SoftwareName,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<SoftwareVersion>,

    pub action: SoftwareRequestUpdateAction,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Sub list of modules grouped by plugin type.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareListResponseList {
    #[serde(rename = "type")]
    pub plugin_type: SoftwareType,
    pub list: Vec<SoftwareListModule>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
pub struct SoftwareListModule {
    #[serde(skip)]
    pub software_type: SoftwareType,
    pub name: SoftwareName,
    pub version: Option<SoftwareVersion>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestUpdateStatus {
    pub update: SoftwareModulesUpdateRequest,
    pub status: UpdateStatus,
}

impl SoftwareRequestUpdateStatus {
    pub fn new(update: &SoftwareModulesUpdateRequest, result: Result<(), SoftwareError>) -> Self {
        let status = match result {
            Ok(()) => UpdateStatus::Success,
            Err(reason) => UpdateStatus::Error { reason },
        };

        Self {
            update: update.clone(),
            status,
        }
    }

    pub fn scheduled(update: &SoftwareModulesUpdateRequest) -> Self {
        Self {
            update: update.clone(),
            status: UpdateStatus::Scheduled,
        }
    }

    pub fn cancelled(update: &SoftwareModulesUpdateRequest) -> Self {
        Self {
            update: update.clone(),
            status: UpdateStatus::Cancelled,
        }
    }
}

/// Variants represent Software Operations Supported actions.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareRequestUpdateAction {
    Install,
    Remove,
}

/// Possible statuses for result of Software operation.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareOperationResultStatus {
    Successful,
    Failed,
    Executing,
}

/// Software Operation Response payload format.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareRequestResponse {
    // TODO: Is this the right approach, maybe nanoid?
    pub id: usize,
    pub status: SoftwareOperationResultStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_software_list: Option<ListSoftwareListResponseList>,

    // TODO: Make it vec and use is_empty instead of Option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failures: Option<ListSoftwareListResponseList>,
}

// TODO: Add methods to to handle response changes, eg add_failure, update reason ...
impl SoftwareRequestResponse {
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

/// List of Software
pub type ListSoftwareListResponseList = Vec<SoftwareListResponseList>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_software_request_list() {
        let request = SoftwareRequestList { id: 1234 };
        let expected_json = r#"{"id":1234}"#;

        let actual_json = request.to_json().expect("Failed to serialize");

        assert_eq!(actual_json, expected_json);

        let de_request =
            SoftwareRequestList::from_json(actual_json.as_str()).expect("failed to deserialize");
        assert_eq!(request, de_request);
    }

    #[test]
    fn serde_software_request_update() {
        let debian_module1 = SoftwareModulesUpdateRequest {
            name: "debian1".into(),
            version: Some("0.0.1".into()),
            action: SoftwareRequestUpdateAction::Install,
            url: None,
        };

        let debian_module2 = SoftwareModulesUpdateRequest {
            name: "debian2".into(),
            version: Some("0.0.2".into()),
            action: SoftwareRequestUpdateAction::Install,
            url: None,
        };

        let debian_list = SoftwareRequestUpdateList {
            plugin_type: "debian".into(),
            list: vec![debian_module1, debian_module2],
        };

        let docker_module1 = SoftwareModulesUpdateRequest {
            name: "docker1".into(),
            version: Some("0.0.1".into()),
            action: SoftwareRequestUpdateAction::Remove,
            url: Some("test.com".into()),
        };

        let docker_list = SoftwareRequestUpdateList {
            plugin_type: "docker".into(),
            list: vec![docker_module1],
        };

        let request = SoftwareRequestUpdate {
            id: 1234,
            update_list: vec![debian_list, docker_list],
        };

        let expected_json = r#"{"id":1234,"updateList":[{"type":"debian","list":[{"name":"debian1","version":"0.0.1","action":"install"},{"name":"debian2","version":"0.0.2","action":"install"}]},{"type":"docker","list":[{"name":"docker1","version":"0.0.1","action":"remove","url":"test.com"}]}]}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request =
            SoftwareRequestUpdate::from_json(&actual_json).expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }

    #[test]
    fn serialize_and_parse_update_status() {
        let status = SoftwareUpdateStatus {
            update: SoftwareUpdate::Install {
                module: SoftwareModule {
                    software_type: "".into(),
                    name: "test_core".into(),
                    version: None,
                    url: None,
                },
            },
            status: UpdateStatus::Success,
        };

        let expected_json =
            r#"{"update":{"action":"install","name":"test_core"},"status":"Success"}"#;
        let actual_json = serde_json::to_string(&status).expect("Fail to serialize a status");
        assert_eq!(actual_json, expected_json);

        let parsed_status: SoftwareUpdateStatus =
            serde_json::from_str(&actual_json).expect("Fail to parse the json status");
        assert_eq!(parsed_status, status);
    }

    #[test]
    fn serde_software_list_empty_successful() {
        let request = SoftwareRequestResponse {
            id: 1234,
            status: SoftwareOperationResultStatus::Successful,
            reason: None,
            current_software_list: Some(vec![]),
            failures: None,
        };

        let expected_json = r#"{"id":1234,"status":"successful","currentSoftwareList":[]}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request = SoftwareRequestResponse::from_json(&actual_json)
            .expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }

    #[test]
    fn serde_software_list_some_modules_successful() {
        let module1 = SoftwareListModule {
            software_type: "".into(),
            name: "debian1".into(),
            version: Some("0.0.1".into()),
        };

        let docker_module1 = SoftwareListResponseList {
            plugin_type: "debian".into(),
            list: vec![module1],
        };

        let request = SoftwareRequestResponse {
            id: 1234,
            status: SoftwareOperationResultStatus::Successful,
            reason: None,
            current_software_list: Some(vec![docker_module1]),
            failures: None,
        };

        let expected_json = r#"{"id":1234,"status":"successful","currentSoftwareList":[{"type":"debian","list":[{"name":"debian1","version":"0.0.1"}]}]}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request = SoftwareRequestResponse::from_json(&actual_json)
            .expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }
}