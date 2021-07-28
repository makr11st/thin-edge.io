use crate::{error::SoftwareError, software::*};
use serde::{Deserialize, Serialize};

pub trait Jsonify<'a>
where
    Self: Deserialize<'a> + Serialize + Sized,
{
    fn from_json(json_str: &'a str) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_str(json_str)?)
    }

    fn from_slice(bytes: &'a [u8]) -> Result<Self, SoftwareError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    fn to_json(&self) -> Result<String, SoftwareError> {
        Ok(serde_json::to_string(self)?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, SoftwareError> {
        Ok(serde_json::to_vec(self)?)
    }
}

/// Message payload definition for SoftwareList request.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareRequestList {
    pub id: usize,
}

impl<'a> Jsonify<'a> for SoftwareRequestList {}

/// Message payload definition for SoftwareUpdate request.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareRequestUpdate {
    pub id: usize,
    pub update_list: Vec<SoftwareRequestResponseSoftwareList>,
}

impl<'a> Jsonify<'a> for SoftwareRequestUpdate {}

/// Sub list of modules grouped by plugin type.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SoftwareRequestResponseSoftwareList {
    #[serde(rename = "type")]
    pub plugin_type: SoftwareType,
    pub list: Vec<SoftwareModuleItem>,
}

/// Possible statuses for result of Software operation.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareOperationStatus {
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
    pub status: SoftwareOperationStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub current_software_list: Vec<SoftwareRequestResponseSoftwareList>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<SoftwareRequestResponseSoftwareList>,
}

impl<'a> Jsonify<'a> for SoftwareRequestResponse {}

// TODO: Add methods to handle response changes, eg add_failure, update reason ...
impl SoftwareRequestResponse {
    pub fn new(id: usize, status: SoftwareOperationStatus) -> Self {
        SoftwareRequestResponse {
            id,
            status,
            current_software_list: vec![],
            reason: None,
            failures: vec![],
        }
    }

    pub fn finalize_response(&mut self, software_list: Vec<SoftwareRequestResponseSoftwareList>) {
        if self.failures.is_empty() {
            self.status = SoftwareOperationStatus::Successful;
        }

        self.current_software_list = software_list;
    }
}

impl Into<SoftwareModule> for SoftwareModuleItem {
    fn into(self) -> SoftwareModule {
        SoftwareModule {
            name: self.name,
            version: self.version,
            url: self.url,
        }
    }
}

impl Into<Option<SoftwareModuleUpdate>> for SoftwareModuleItem {
    fn into(self) -> Option<SoftwareModuleUpdate> {
        match self.action {
            Some(SoftwareModuleAction::Install) => Some(SoftwareModuleUpdate::Install {
                module: self.into(),
            }),
            Some(SoftwareModuleAction::Remove) => Some(SoftwareModuleUpdate::Remove {
                module: self.into(),
            }),
            None => None,
        }
    }
}

impl From<SoftwareModule> for SoftwareModuleItem {
    fn from(module: SoftwareModule) -> Self {
        SoftwareModuleItem {
            name: module.name,
            version: module.version,
            url: module.url,
            action: None,
            reason: None,
        }
    }
}

impl From<SoftwareModuleUpdate> for SoftwareModuleItem {
    fn from(update: SoftwareModuleUpdate) -> Self {
        match update {
            SoftwareModuleUpdate::Install { module } => SoftwareModuleItem {
                name: module.name,
                version: module.version,
                url: module.url,
                action: Some(SoftwareModuleAction::Install),
                reason: None,
            },
            SoftwareModuleUpdate::Remove { module } => SoftwareModuleItem {
                name: module.name,
                version: module.version,
                url: module.url,
                action: Some(SoftwareModuleAction::Remove),
                reason: None,
            },
        }
    }
}

impl From<SoftwareModuleUpdateResult> for SoftwareModuleItem {
    fn from(result: SoftwareModuleUpdateResult) -> Self {
        let mut msg: SoftwareModuleItem = result.update.into();
        msg.reason = result.error.map(|err| format!("{}", err));
        msg
    }
}

#[cfg(test)]
mod tests {

    use crate::software::SoftwareModuleAction;

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
        let debian_module1 = SoftwareModuleItem {
            name: "debian1".into(),
            version: Some("0.0.1".into()),
            action: Some(SoftwareModuleAction::Install),
            url: None,
            reason: None,
        };

        let debian_module2 = SoftwareModuleItem {
            name: "debian2".into(),
            version: Some("0.0.2".into()),
            action: Some(SoftwareModuleAction::Install),
            url: None,
            reason: None,
        };

        let debian_list = SoftwareRequestResponseSoftwareList {
            plugin_type: "debian".into(),
            list: vec![debian_module1, debian_module2],
        };

        let docker_module1 = SoftwareModuleItem {
            name: "docker1".into(),
            version: Some("0.0.1".into()),
            action: Some(SoftwareModuleAction::Remove),
            url: Some("test.com".into()),
            reason: None,
        };

        let docker_list = SoftwareRequestResponseSoftwareList {
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

    // #[test]
    // fn serialize_and_parse_update_status() {
    //     let status = SoftwareUpdateStatus {
    //         update: SoftwareUpdate::Install {
    //             module: SoftwareModule {
    //                 software_type: "".into(),
    //                 name: "test_core".into(),
    //                 version: None,
    //                 url: None,
    //             },
    //         },
    //         status: UpdateStatus::Success,
    //     };

    //     let expected_json =
    //         r#"{"update":{"action":"install","name":"test_core"},"status":"Success"}"#;
    //     let actual_json = serde_json::to_string(&status).expect("Fail to serialize a status");
    //     assert_eq!(actual_json, expected_json);

    //     let parsed_status: SoftwareUpdateStatus =
    //         serde_json::from_str(&actual_json).expect("Fail to parse the json status");
    //     assert_eq!(parsed_status, status);
    // }

    #[test]
    fn serde_software_list_empty_successful() {
        let request = SoftwareRequestResponse {
            id: 1234,
            status: SoftwareOperationStatus::Successful,
            reason: None,
            current_software_list: vec![],
            failures: vec![],
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
        let module1 = SoftwareModuleItem {
            name: "debian1".into(),
            version: Some("0.0.1".into()),
            action: None,
            url: None,
            reason: None,
        };

        let docker_module1 = SoftwareRequestResponseSoftwareList {
            plugin_type: "debian".into(),
            list: vec![module1],
        };

        let request = SoftwareRequestResponse {
            id: 1234,
            status: SoftwareOperationStatus::Successful,
            reason: None,
            current_software_list: vec![docker_module1],
            failures: vec![],
        };

        let expected_json = r#"{"id":1234,"status":"successful","currentSoftwareList":[{"type":"debian","list":[{"name":"debian1","version":"0.0.1"}]}]}"#;

        let actual_json = request.to_json().expect("Fail to serialize the request");
        assert_eq!(actual_json, expected_json);

        let parsed_request = SoftwareRequestResponse::from_json(&actual_json)
            .expect("Fail to parse the json request");
        assert_eq!(parsed_request, request);
    }
}