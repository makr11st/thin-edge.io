use std::{process, time::Duration};

use crate::core::{converter::*, error::*};

use mqtt_channel::{
    Connection, Message, MqttError, SinkExt, StreamExt, Topic, TopicFilter, UnboundedReceiver,
    UnboundedSender,
};
use serde_json::json;
use tracing::{error, info, instrument};

const SYNC_WINDOW: Duration = Duration::from_secs(3);

pub async fn create_mapper(
    app_name: &str,
    mqtt_host: String,
    mqtt_port: u16,
    converter: Box<dyn Converter<Error = ConversionError>>,
) -> Result<Mapper, anyhow::Error> {
    info!("{} starting", app_name);

    let health_check_topics: TopicFilter = vec![
        "tedge/health-check",
        format!("tedge/health-check/{}", app_name).as_str(),
    ]
    .try_into()
    .expect("health check topics must be valid");

    let health_status_topic = Topic::new_unchecked(format!("tedge/health/{}", app_name).as_str());

    let mapper_config = converter.get_mapper_config();
    let mut topic_filter = mapper_config.in_topic_filter.clone();
    topic_filter.add_all(health_check_topics.clone());

    let mqtt_client =
        Connection::new(&mqtt_config(app_name, &mqtt_host, mqtt_port, topic_filter)?).await?;

    Mapper::subscribe_errors(mqtt_client.errors);

    Ok(Mapper::new(
        mqtt_client.received,
        mqtt_client.published,
        converter,
        health_check_topics,
        health_status_topic,
    ))
}

pub fn mqtt_config(
    name: &str,
    host: &str,
    port: u16,
    topic_filter: TopicFilter,
) -> Result<mqtt_channel::Config, anyhow::Error> {
    Ok(mqtt_channel::Config::default()
        .with_host(host)
        .with_port(port)
        .with_session_name(name)
        .with_subscriptions(topic_filter)
        .with_max_packet_size(10 * 1024 * 1024))
}

pub struct Mapper {
    input: UnboundedReceiver<Message>,
    output: UnboundedSender<Message>,
    converter: Box<dyn Converter<Error = ConversionError>>,
    health_check_topics: TopicFilter,
    health_status_topic: Topic,
}

impl Mapper {
    pub fn new(
        input: UnboundedReceiver<Message>,
        output: UnboundedSender<Message>,
        converter: Box<dyn Converter<Error = ConversionError>>,
        health_check_topics: TopicFilter,
        health_status_topic: Topic,
    ) -> Self {
        Self {
            input,
            output,
            converter,
            health_check_topics,
            health_status_topic,
        }
    }

    pub(crate) async fn run(&mut self) -> Result<(), MqttError> {
        info!("Running");
        self.process_messages().await?;
        Ok(())
    }

    #[instrument(skip(errors), name = "errors")]
    fn subscribe_errors(mut errors: UnboundedReceiver<MqttError>) {
        tokio::spawn(async move {
            while let Some(error) = errors.next().await {
                error!("{}", error);
            }
        });
    }

    #[instrument(skip(self), name = "messages")]
    async fn process_messages(&mut self) -> Result<(), MqttError> {
        let init_messages = self.converter.init_messages();
        for init_message in init_messages.into_iter() {
            let _ = self.output.send(init_message).await;
        }

        // Start the sync phase here and process messages until the sync window times out
        let _ = tokio::time::timeout(SYNC_WINDOW, async {
            while let Some(message) = self.input.next().await {
                self.process_message(message).await;
            }
        })
        .await;

        // Once the sync phase is complete, retrieve all sync messages from the converter and process them
        let sync_messages = self.converter.sync_messages();
        for message in sync_messages {
            self.process_message(message).await;
        }

        // Continue processing messages after the sync period
        while let Some(message) = self.input.next().await {
            self.process_message(message).await;
        }

        Ok(())
    }

    async fn process_message(&mut self, message: Message) {
        if self.health_check_topics.accept(&message) {
            let health_status = json!({
                "status": "up",
                "pid": process::id()
            })
            .to_string();
            let health_message = Message::new(&self.health_status_topic, health_status);
            let _ = self.output.send(health_message).await;
        } else {
            let converted_messages = self.converter.convert(&message).await;
            for converted_message in converted_messages.into_iter() {
                let _ = self.output.send(converted_message).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_include;
    use async_trait::async_trait;
    use mqtt_channel::{Message, Topic, TopicFilter};
    use serde_json::Value;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    #[serial_test::serial]
    async fn a_valid_input_leads_to_a_translated_output() -> Result<(), anyhow::Error> {
        // Given an MQTT broker
        let broker = mqtt_tests::test_mqtt_broker();

        // Given a mapper
        let name = "mapper_under_test";
        let mut mapper = create_mapper(
            name,
            "localhost".into(),
            broker.port,
            Box::new(UppercaseConverter::new()),
        )
        .await?;

        // Let's run the mapper in the background
        tokio::spawn(async move {
            let _ = mapper.run().await;
        });
        sleep(Duration::from_secs(1)).await;

        // One can now send requests
        let timeout = Duration::from_secs(1);

        // Happy path
        let input = "abcde";
        let expected = Some("ABCDE".to_string());
        let actual = broker
            .wait_for_response_on_publish("in_topic", input, "out_topic", timeout)
            .await;
        assert_eq!(expected, actual);

        // Ill-formed input
        let input = "éèê";
        let expected = Some(format!("{}", UppercaseConverter::conversion_error()));
        let actual = broker
            .wait_for_response_on_publish("in_topic", input, "err_topic", timeout)
            .await;
        assert_eq!(expected, actual);

        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn health_check() -> Result<(), anyhow::Error> {
        // Given an MQTT broker
        let broker = mqtt_tests::test_mqtt_broker();

        // Given a mapper
        let name = "mapper_under_test";

        let mut mapper = create_mapper(
            name,
            "localhost".to_string(),
            broker.port,
            Box::new(UppercaseConverter::new()),
        )
        .await?;

        // Let's run the mapper in the background
        tokio::spawn(async move {
            let _ = mapper.run().await;
        });
        sleep(Duration::from_secs(1)).await;

        let health_check_topic = format!("tedge/health-check/{name}");
        let health_topic = format!("tedge/health/{name}");
        let health_status = broker
            .wait_for_response_on_publish(
                &health_check_topic,
                "",
                &health_topic,
                Duration::from_secs(1),
            )
            .await
            .expect("JSON status message");
        let health_status: Value = serde_json::from_str(health_status.as_str())?;
        assert_json_include!(actual: &health_status, expected: json!({"status": "up"}));
        assert!(health_status["pid"].is_number());

        let common_health_check_topic = "tedge/health-check";
        let health_status = broker
            .wait_for_response_on_publish(
                &common_health_check_topic,
                "",
                &health_topic,
                Duration::from_secs(1),
            )
            .await
            .expect("JSON status message");
        let health_status: Value = serde_json::from_str(health_status.as_str())?;
        assert_json_include!(actual: &health_status, expected: json!({"status": "up"}));
        assert!(health_status["pid"].is_number());

        Ok(())
    }

    struct UppercaseConverter {
        mapper_config: MapperConfig,
    }

    impl UppercaseConverter {
        pub fn new() -> UppercaseConverter {
            let mapper_config = MapperConfig {
                in_topic_filter: TopicFilter::new("in_topic").expect("invalid topic filter"),
                out_topic: Topic::new_unchecked("out_topic"),
                errors_topic: Topic::new_unchecked("err_topic"),
            };
            UppercaseConverter { mapper_config }
        }

        pub fn conversion_error() -> ConversionError {
            // Just a stupid error that matches the expectations of the mapper
            ConversionError::FromMapper(MapperError::HomeDirNotFound)
        }
    }

    #[async_trait]
    impl Converter for UppercaseConverter {
        type Error = ConversionError;

        fn get_mapper_config(&self) -> &MapperConfig {
            &self.mapper_config
        }

        async fn try_convert(&mut self, input: &Message) -> Result<Vec<Message>, Self::Error> {
            let input = input.payload_str().expect("utf8");
            if input.is_ascii() {
                let msg = vec![Message::new(
                    &self.mapper_config.out_topic,
                    input.to_uppercase(),
                )];
                Ok(msg)
            } else {
                Err(UppercaseConverter::conversion_error())
            }
        }
    }
}
