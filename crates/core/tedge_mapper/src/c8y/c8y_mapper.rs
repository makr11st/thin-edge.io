use crate::{
    c8y::{c8y_converter::CumulocityConverter, http_proxy::JwtAuthHttpProxy},
    mapping::{
        component::TEdgeComponent, mapper::create_mapper, operations::Operations,
        size_threshold::SizeThreshold,
    },
};

use async_trait::async_trait;
use tedge_config::{
    ConfigSettingAccessor, DeviceIdSetting, DeviceTypeSetting, MqttPortSetting, TEdgeConfig,
};
use tracing::{info_span, Instrument};

const CUMULOCITY_MAPPER_NAME: &str = "tedge-mapper-c8y";

pub struct CumulocityMapper {}

impl CumulocityMapper {
    pub fn new() -> CumulocityMapper {
        CumulocityMapper {}
    }
}

#[async_trait]
impl TEdgeComponent for CumulocityMapper {
    async fn start(&self, tedge_config: TEdgeConfig) -> Result<(), anyhow::Error> {
        let size_threshold = SizeThreshold(16 * 1024);

        let operations = Operations::try_new("/etc/tedge/operations", "c8y")?;
        let http_proxy = JwtAuthHttpProxy::try_new(&tedge_config).await?;

        let converter = Box::new(CumulocityConverter::new(
            size_threshold,
            &operations,
            &http_proxy,
        ));

        let converter = Box::new(CumulocityConverter::new(
            size_threshold,
            device_name,
            device_type,
        ));

        let mut mapper = create_mapper(CUMULOCITY_MAPPER_NAME, mqtt_port, converter).await?;

        mapper
            .run()
            .instrument(info_span!(CUMULOCITY_MAPPER_NAME))
            .await?;

        Ok(())
    }
}
