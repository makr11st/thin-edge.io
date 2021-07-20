use crate::{error::AgentError, state::AgentStateRepository};
use log::{debug, error, info};
use mqtt_client::{Client, Message, MqttClient, Topic, TopicFilter};
use std::sync::Arc;
use tedge_config::TEdgeConfigLocation;
use tedge_sm_lib::{message::*, plugin::*, plugin_manager::*, software::*};
use tedge_users::{UserManager, ROOT_USER};

#[derive(Debug)]
pub struct SmAgentConfig {
    pub request_topic: TopicFilter,
    pub response_topic_list: Topic,
    pub response_topic_update: Topic,
    pub errors_topic: Topic,
    pub mqtt_client_config: mqtt_client::Config,
}

impl Default for SmAgentConfig {
    fn default() -> Self {
        let request_topic =
            TopicFilter::new("tedge/commands/req/software/#").expect("Invalid topic");

        let response_topic_list =
            Topic::new("tedge/commands/res/software/list").expect("Invalid topic");

        let response_topic_update =
            Topic::new("tedge/commands/res/software/update").expect("Invalid topic");

        let errors_topic = Topic::new("tedge/errors").expect("Invalid topic");

        let mqtt_client_config = mqtt_client::Config::default().with_packet_size(50 * 1024);

        Self {
            request_topic,
            response_topic_list,
            response_topic_update,
            errors_topic,
            mqtt_client_config,
        }
    }
}

#[derive(Debug)]
pub struct SmAgent {
    config: SmAgentConfig,
    name: String,
    user_manager: UserManager,
    config_location: TEdgeConfigLocation,
}

impl SmAgent {
    pub fn new(
        name: &str,
        user_manager: UserManager,
        config_location: TEdgeConfigLocation,
    ) -> Self {
        Self {
            config: SmAgentConfig::default(),
            name: name.into(),
            user_manager,
            config_location,
        }
    }

    pub async fn start(&self) -> Result<(), AgentError> {
        info!("Starting sm-agent");

        let plugins = Arc::new(ExternalPlugins::open("/etc/tedge/sm-plugins")?);
        if plugins.empty() {
            error!("Couldn't load plugins from /etc/tedge/sm-plugins");
            return Err(AgentError::NoPlugins);
        }

        let mqtt = Client::connect(self.name.as_str(), &self.config.mqtt_client_config).await?;
        let mut errors = mqtt.subscribe_errors();
        tokio::spawn(async move {
            while let Some(error) = errors.next().await {
                error!("{}", error);
            }
        });

        // let persistance_store = AgentStateRepository::new();
        // * Maybe it would be nice if mapper/registry responds
        let () = publish_capabilities(&mqtt).await?;

        let () = self.subscribe_and_process(&mqtt, &plugins).await?;

        Ok(())
    }

    async fn subscribe_and_process(
        &self,
        mqtt: &Client,
        plugins: &Arc<ExternalPlugins>,
    ) -> Result<(), AgentError> {
        let mut operations = mqtt.subscribe(self.config.request_topic.clone()).await?;
        while let Some(message) = operations.next().await {
            info!("Request {:?}", message);

            let operation: SoftwareOperation = message.topic.clone().into();
            dbg!(&operation);

            match operation {
                SoftwareOperation::CurrentSoftwareList => {
                    self.handle_software_list_request(
                        mqtt,
                        plugins.clone(),
                        &self.config.response_topic_list,
                        &message,
                    )
                    .await?;
                }

                SoftwareOperation::SoftwareUpdates => {
                    self.handle_software_update_request(
                        mqtt,
                        plugins.clone(),
                        &self.config.response_topic_update,
                        &message,
                    )
                    .await?;
                }

                SoftwareOperation::UnknownOperation => self.handle_unknown_operation(),
            }
        }

        Ok(())
    }

    fn handle_unknown_operation(&self) {
        todo!()
    }

    async fn handle_software_update_request(
        &self,
        mqtt: &Client,
        plugins: Arc<ExternalPlugins>,
        response_topic: &Topic,
        message: &Message,
    ) -> Result<(), AgentError> {
        let request = match SoftwareRequestUpdate::from_slice(message.payload_trimmed()) {
            Ok(request) => {
                let () = self
                    .publish_status_executing(mqtt, response_topic, request.id)
                    .await?;
                request
            }

            Err(error) => {
                error!("Parsing error: {}", error);
                let _ = mqtt
                    .publish(Message::new(response_topic, format!("{}", error)))
                    .await?;

                return Err(SoftwareError::ParseError {
                    reason: "Parsing failed".into(),
                }
                .into());
            }
        };

        let mut response = SoftwareResponseUpdateStatus {
            id: request.id,
            status: SoftwareOperationResultStatus::Failed,
            reason: None,
            current_software_list: None,
            failures: None,
        };

        let mut failures = ListSoftwareListResponseList::new();

        let plugins = plugins.clone();
        for software_list_type in request.update_list {
            let plugin = plugins
                .by_software_type(&software_list_type.plugin_type)
                .unwrap();

            if let Err(e) = plugin.prepare() {
                response.reason = Some(format!("Failed prepare stage: {}", e));

                let _ = mqtt
                    .publish(Message::new(response_topic, response.to_bytes()?))
                    .await?;
            };

            let mut failures_modules = Vec::new();

            let () = self.install_or_remove(
                software_list_type,
                plugin,
                &mut response,
                &mut failures_modules,
            )?;

            let () = plugin.finalize()?;

            failures.push(SoftwareListResponseList {
                plugin_type: plugin.name.clone(),
                list: failures_modules,
            });
        }

        let software_list = tokio::task::spawn_blocking(move || plugins.list()).await??;
        let () = self.finalize_response(&mut response, &software_list, &failures)?;

        let _ = mqtt
            .publish(Message::new(response_topic, response.to_bytes()?))
            .await?;

        Ok(())
    }

    fn install_or_remove(
        &self,
        software_list_type: SoftwareRequestUpdateList,
        plugin: &ExternalPluginCommand,
        response: &mut SoftwareResponseUpdateStatus,
        failures_modules: &mut Vec<SoftwareListModule>,
    ) -> Result<(), AgentError> {
        for module in software_list_type.list.into_iter() {
            match module.action {
                SoftwareRequestUpdateAction::Install => {
                    let _user_guard = self.user_manager.become_user(ROOT_USER)?;

                    if let Err(_err) = plugin.install(&module) {
                        response.reason = Some("Module installation failed".into());
                        let () = failures_modules.push(SoftwareListModule {
                            software_type: module.name.clone(),
                            name: module.name,
                            version: module.version,
                        });
                    }
                }

                SoftwareRequestUpdateAction::Remove => {
                    let _user_guard = self.user_manager.become_user(ROOT_USER)?;

                    if let Err(_err) = plugin.remove(&module) {
                        response.reason = Some("Module removal failed".into());
                        let () = failures_modules.push(SoftwareListModule {
                            software_type: module.name.clone(),
                            name: module.name,
                            version: module.version,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn finalize_response(
        &self,
        response: &mut SoftwareResponseUpdateStatus,
        software_list: &[SoftwareListResponseList],
        failures: &[SoftwareListResponseList],
    ) -> Result<(), AgentError> {
        if failures.is_empty() {
            response.status = SoftwareOperationResultStatus::Successful;
        }

        response.current_software_list = Some(software_list.to_vec());
        response.failures = Some(failures.to_vec());
        Ok(())
    }

    async fn publish_status_executing(
        &self,
        mqtt: &Client,
        response_topic: &Topic,
        id: usize,
    ) -> Result<(), AgentError> {
        let response = SoftwareResponseUpdateStatus {
            id,
            status: SoftwareOperationResultStatus::Executing,
            current_software_list: None,
            reason: None,
            failures: None,
        };

        let _ = mqtt
            .publish(Message::new(response_topic, response.to_bytes()?))
            .await?;

        Ok(())
    }

    async fn handle_software_list_request(
        &self,
        mqtt: &Client,
        plugins: Arc<ExternalPlugins>,
        response_topic: &Topic,
        message: &Message,
    ) -> Result<(), AgentError> {
        let software_list = tokio::task::spawn_blocking(move || plugins.list()).await??;

        let request = match SoftwareRequestList::from_slice(message.payload_trimmed()) {
            Ok(request) => request,
            Err(error) => {
                debug!("Parsing error: {}", error);
                let _ = mqtt
                    .publish(Message::new(response_topic, format!("{}", error)))
                    .await?;

                return Err(SoftwareError::ParseError {
                    reason: "Parsing Error".into(),
                }
                .into());
            }
        };

        let response = SoftwareListResponse {
            id: request.id,
            status: SoftwareOperationResultStatus::Successful,
            list: software_list,
        };

        let _ = mqtt
            .publish(Message::new(response_topic, response.to_bytes()?))
            .await?;
        Ok(())
    }
}

async fn publish_capabilities(mqtt: &Client) -> Result<(), AgentError> {
    mqtt.publish(Message::new(&Topic::new("tedge/capabilities/software/list")?, "").retain())
        .await?;

    mqtt.publish(Message::new(&Topic::new("tedge/capabilities/software/update")?, "").retain())
        .await?;

    Ok(())
}
