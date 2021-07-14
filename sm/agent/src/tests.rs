use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use once_cell::sync::Lazy;
use librumqttd::{async_locallink, ConnectionSettings, ConsoleSettings, ServerSettings};
use tokio::sync::once_cell;
// static SERVER: Lazy<Future>


#[tokio::test]
async fn true_test_name() -> anyhow::Result<()> {
    let mqtt_server_handle = tokio::spawn(async { start_broker().await });

    // Call abort to stop the server and finish the test.
    mqtt_server_handle.abort();

    Ok(())
}

async fn start_broker() -> anyhow::Result<()> {
    let (mut router, _console, servers, _builder) =
        async_locallink::construct_broker(get_rumqttd_config());

    let router = tokio::task::spawn_blocking(move || {
        router.start().unwrap();
    });

    servers.await;
    Ok(router.await?)
}

fn get_rumqttd_config() -> librumqttd::Config {
    let router_config = rumqttlog::Config {
        id: 0,
        dir: "/tmp/rumqttd".into(),
        max_segment_size: 10240,
        max_segment_count: 10,
        max_connections: 10,
    };

    let connections_settings = ConnectionSettings {
        connection_timeout_ms: 1,
        max_client_id_len: 256,
        throttle_delay_ms: 0,
        max_payload_size: 10240,
        max_inflight_count: 200,
        max_inflight_size: 1024,
        username: None,
        password: None,
    };

    let server_config = ServerSettings {
        listen: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 1884)),
        cert: None,
        next_connection_delay_ms: 1,
        connections: connections_settings,
    };

    let mut servers = HashMap::new();
    servers.insert("1".to_string(), server_config);

    let console_settings = ConsoleSettings {
        listen: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3030)),
    };

    librumqttd::Config {
        id: 0,
        router: router_config,
        servers,
        cluster: None,
        replicator: None,
        console: console_settings,
    }
}
