use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::time;

use crate::modbus_reader::read_current;

const MODBUS_ADDR: &str = "10.234.1.82:502";
const MQTT_TOPIC: &str = "be-strom/data";
const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// Connects to the MQTT broker configured via `MQTT_HOST` / `MQTT_PORT`
/// and publishes a JSON snapshot of `DeviceData` every 3 seconds.
///
/// # Environment variables
/// | Variable    | Default       | Description                   |
/// |-------------|---------------|-------------------------------|
/// | `MQTT_HOST` | `localhost`   | Hostname or IP of the broker  |
/// | `MQTT_PORT` | `1883`        | TCP port of the broker        |
///
/// The payload is the `DeviceData` struct serialised as a flat JSON object,
/// e.g.:
/// ```json
/// {
///   "device": "UR20-3EM...",
///   "voltage":   {"value": 230.123, "unit": "V"},
///   "current_1": {"value":   1.234, "unit": "A"},
///   ...
/// }
/// ```
pub async fn run() -> anyhow::Result<()> {
    let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1883);

    println!("MQTT broker: {}:{}", host, port);
    println!(
        "Publishing to topic '{}' every {}s",
        MQTT_TOPIC,
        POLL_INTERVAL.as_secs()
    );

    let mut mqtt_opts = MqttOptions::new("be-strom-publisher", &host, port);
    mqtt_opts.set_keep_alive(Duration::from_secs(30));
    // Buffer up to 10 outgoing messages before applying back-pressure.
    let (client, mut event_loop) = AsyncClient::new(mqtt_opts, 10);

    // Drive the event loop on a separate task so that `publish` calls never block.
    tokio::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("MQTT event loop error: {e}");
                    // Brief pause to avoid a tight error loop before reconnect.
                    time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });

    let mut interval = time::interval(POLL_INTERVAL);

    loop {
        interval.tick().await;

        match read_current(MODBUS_ADDR).await {
            Ok(data) => match serde_json::to_string(&data) {
                Ok(payload) => {
                    if let Err(e) = client
                        .publish(MQTT_TOPIC, QoS::AtLeastOnce, false, payload.as_bytes())
                        .await
                    {
                        eprintln!("MQTT publish error: {e}");
                    } else {
                        println!("Published: {payload}");
                    }
                }
                Err(e) => eprintln!("JSON serialisation error: {e}"),
            },
            Err(e) => eprintln!("Modbus read error: {e}"),
        }
    }
}
