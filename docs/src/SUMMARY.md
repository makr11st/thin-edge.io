# Summary

- [Summary](./SUMMARY.md)

- [Introduction](001_overview.md)

- [User Documentation](user_doc.md)

  - [Tutorials](tutorials/README.md)

    - [Connect my device to Cumulocity IoT](./tutorials/connect-c8y.md)
    - [Connect my device to Azure IoT](./tutorials/connect-azure.md)
    - [Send Thin Edge Json data](./tutorials/send-thin-edge-data.md)
    - [Raise alarms](./tutorials/raise-alarm.md)
    - [Send events](./tutorials/send-events.md)
    - [Monitor my device](./tutorials/device-monitoring.md)
    - [Manage my device software](./tutorials/software-management.md)
    - [Write my software management plugin](./tutorials/write-my-software-management-plugin.md)
    - [Supported Operations Management for Cumulocity IoT](./tutorials/supported_operations.md)

  - [How-to Guides](howto-guides/README.md)
    - [Installation](howto-guides/002_installation.md)
    - [How to create a test certificate](./howto-guides/003_registration.md)
    - [How to connect a cloud end-point](./howto-guides/004_connect.md)
    - [How to send MQTT messages](./howto-guides/005_pub_sub.md)
    - [How to test the cloud connection?](./howto-guides/007_test_connection.md)
    - [How to configure the local mqtt bind address and port](./howto-guides/008_config_local_mqtt_bind_address_and_port.md)
    - [How to trouble shoot device monitoring](./howto-guides/009_trouble_shooting_monitoring.md)
    - [How to add self-signed certificate root to trusted certificates list?](./howto-guides/010_add_self_signed_trusted.md)
    - [How to retrieve JWT token from Cumulocity?](./howto-guides/011_retrieve_jwt_token_from_cumulocity.md)
    - [How to install and enable software management?](./howto-guides/012_install_and_enable_software_management.md)
    - [How to connect an external device?](./howto-guides/013_connect_external_device.md)
    - [How to access the logs on the device?](./howto-guides/014_thin_edge_logs.md)
    - [How to install thin-edge.io on any Linux OS (no deb support)?](./howto-guides/015_installation_without_deb_support.md)
    - [How to restart your thin-edge.io device](./howto-guides/016_restart_device_operation.md)
    - [How to use apama software management plugin](./howto-guides/017_apama_software_management_plugin.md)
    - [How to change temp path](./howto-guides/018_change_temp_path.md)
    - [How to use thin-edge.io with your preferred init system](./howto-guides/019_how_to_use_preferred_init_system.md)
    - [How to monitor health of tedge daemons](./howto-guides/020_monitor_tedge_health.md)
    - [How to enable systemd watchdog monitoring for tedge services?](./howto-guides/021_enable_tedge_watchdog_using_systemd.md)
    - [How to add custom fragments to Cumulocity](./howto-guides/022_c8y_fragments.md)
    - [How to retrieve logs with the log plugin](./howto-guides/023_c8y_log_plugin.md)
    - [How to use Cumulocity Custom SmartREST 2.0 Templates with `thin-edge.io`](./howto-guides/024_smartrest_templates.md)

- [Developer Documentation](dev_doc.md)

  - [Architecture](architecture/README.md)

    - [Thin Edge Json](architecture/thin-edge-json.md)
    - [The Mapper](architecture/mapper.md)
    - [Software Management](architecture/software-management.md)
    - [Architecture FAQ](architecture/faq.md)
    - [Platform support](supported-platforms.md)
    - [Init System configuration](references/init-system-config.md)

  - [Write my own software management plugin](./tutorials/write-my-software-management-plugin.md)
  - [Device Configuration Management using Cumulocity](./references/c8y-configuration-management.md)

  - [APIs](api.md)

    - [The Bridged Topics](./references/bridged-topics.md)
    - [The Software Management Plugin API](./references/plugin-api.md)

  - [Building](./BUILDING.md)

- [Command Line Reference](references/README.md)
  - [The `tedge` command](./references/tedge.md)
  - [The `tedge config` command](./references/tedge-config.md)
  - [The `tedge cert` command](./references/tedge-cert.md)
  - [The `tedge connect` command](./references/tedge-connect.md)
  - [The `tedge disconnect` command](./references/tedge-disconnect.md)
  - [The `tedge mqtt` command](./references/tedge-mqtt.md)
