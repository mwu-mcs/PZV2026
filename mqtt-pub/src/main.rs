use rand::{Rng, RngExt};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Serialize;
use std::time::Duration;
use tokio::time;

// ── Configuration ─────────────────────────────────────────────────────────────

struct Config {
    host: String,
    port: u16,
    client_id: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            host: std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1883".to_string())
                .parse()
                .expect("MQTT_PORT must be a valid u16"),
            client_id: std::env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "room-sensor-mock".to_string()),
        }
    }
}

// ── Sensor payload ────────────────────────────────────────────────────────────

/// Combined JSON payload published to `home/room/sensor`.
#[derive(Serialize)]
struct SensorReading {
    /// Temperature in degrees Celsius, rounded to one decimal place.
    temperature_c: f64,
    /// Relative humidity in percent, rounded to one decimal place.
    humidity_pct: f64,
}

impl SensorReading {
    fn new(temperature_c: f64, humidity_pct: f64) -> Self {
        Self {
            temperature_c: round1(temperature_c),
            humidity_pct: round1(humidity_pct),
        }
    }
}

/// Round `v` to one decimal place.
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

// ── Sensor state (random walk) ─────────────────────────────────────────────────

struct Sensor {
    temperature: f64,
    humidity: f64,
}

impl Sensor {
    /// Start close to a comfortable room environment.
    fn new() -> Self {
        Self {
            temperature: 21.0,
            humidity: 55.0,
        }
    }

    /// Nudge both values by a small random amount and clamp to plausible ranges.
    fn tick(&mut self, rng: &mut impl Rng) {
        self.temperature += rng.random_range(-0.3..=0.3);
        self.temperature = self.temperature.clamp(15.0, 30.0);

        self.humidity += rng.random_range(-0.5..=0.5);
        self.humidity = self.humidity.clamp(20.0, 90.0);
    }

    fn reading(&self) -> SensorReading {
        SensorReading::new(self.temperature, self.humidity)
    }
}

// ── MQTT event loop ───────────────────────────────────────────────────────────

/// Drains the `rumqttc` event loop in a background task, logging notable events.
async fn run_event_loop(mut event_loop: rumqttc::EventLoop) {
    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                println!("[mqtt] connected to broker");
            }
            Ok(Event::Incoming(Packet::PubAck(ack))) => {
                println!("[mqtt] ack  pkid={}", ack.pkid);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[mqtt] connection error: {e} – retrying in 5 s…");
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cfg = Config::from_env();

    println!(
        "[mqtt] connecting to {}:{} as client \"{}\"",
        cfg.host, cfg.port, cfg.client_id
    );

    let mut mqtt_opts = MqttOptions::new(&cfg.client_id, &cfg.host, cfg.port);
    mqtt_opts.set_keep_alive(Duration::from_secs(30));

    let (client, event_loop) = AsyncClient::new(mqtt_opts, 16);

    // Drive the event loop in the background so ACKs are processed.
    tokio::spawn(run_event_loop(event_loop));

    let mut rng = rand::rng();
    let mut sensor = Sensor::new();
    let mut interval = time::interval(Duration::from_secs(3));

    println!("[sensor] starting publish loop (every 3 s)");

    loop {
        interval.tick().await;

        sensor.tick(&mut rng);
        let reading = sensor.reading();

        let temp_str = reading.temperature_c.to_string();
        let hum_str = reading.humidity_pct.to_string();
        let json = serde_json::to_string(&reading).expect("serialization failed");

        // Individual topics --------------------------------------------------
        publish(&client, "home/room/temperature", &temp_str).await;
        publish(&client, "home/room/humidity", &hum_str).await;

        // Combined JSON topic ------------------------------------------------
        publish(&client, "home/room/sensor", &json).await;

        println!(
            "[sensor] published  temperature={:.1} °C  humidity={:.1} %",
            reading.temperature_c, reading.humidity_pct
        );
    }
}

/// Publish a UTF-8 `payload` to `topic` with QoS 1 (at least once).
async fn publish(client: &AsyncClient, topic: &str, payload: &str) {
    if let Err(e) = client
        .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
        .await
    {
        eprintln!("[mqtt] failed to publish to {topic}: {e}");
    }
}
