use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Shared state between the MQTT background thread and the egui render loop
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SensorData {
    temperature_c: Option<f32>,
    humidity_pct: Option<f32>,
    last_update: Option<Instant>,
    status: String,
}

// ---------------------------------------------------------------------------
// Deserialised shape of the MQTT payload
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SensorReading {
    temperature_c: f32,
    humidity_pct: f32,
}

// ---------------------------------------------------------------------------
// egui application
// ---------------------------------------------------------------------------

struct DashboardApp {
    shared: Arc<Mutex<SensorData>>,
}

impl DashboardApp {
    fn new(shared: Arc<Mutex<SensorData>>) -> Self {
        Self { shared }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drive "Xs ago" counter without waiting for new MQTT messages.
        ctx.request_repaint_after(Duration::from_millis(500));

        // Snapshot shared state – hold the lock for as short a time as possible.
        let (status, temperature_c, humidity_pct, last_update) = {
            let data = self.shared.lock().unwrap();
            (
                data.status.clone(),
                data.temperature_c,
                data.humidity_pct,
                data.last_update,
            )
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Status bar ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let dot_color = if status == "Connected" {
                    egui::Color32::from_rgb(80, 200, 100)
                } else {
                    egui::Color32::from_rgb(220, 80, 80)
                };
                ui.colored_label(dot_color, "●");
                ui.label(egui::RichText::new(&status).small());
            });

            ui.separator();
            ui.add_space(10.0);

            // ── Metric panels ───────────────────────────────────────────────
            ui.columns(2, |cols| {
                // Temperature
                cols[0].group(|ui| {
                    ui.set_min_size(egui::vec2(190.0, 140.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("🌡 Temperature").size(16.0).strong());
                        ui.add_space(10.0);
                        let text = match temperature_c {
                            Some(t) => format!("{t:.1} °C"),
                            None => "—".to_owned(),
                        };
                        ui.label(egui::RichText::new(text).size(44.0).strong());
                        ui.add_space(6.0);
                    });
                });

                // Humidity
                cols[1].group(|ui| {
                    ui.set_min_size(egui::vec2(190.0, 140.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("💧 Humidity").size(16.0).strong());
                        ui.add_space(10.0);
                        let text = match humidity_pct {
                            Some(h) => format!("{h:.1} %"),
                            None => "—".to_owned(),
                        };
                        ui.label(egui::RichText::new(text).size(44.0).strong());
                        ui.add_space(6.0);
                    });
                });
            });

            ui.add_space(10.0);
            ui.separator();

            // ── Last-update footer ──────────────────────────────────────────
            let footer = match last_update {
                Some(ts) => format!("Last update: {}s ago", ts.elapsed().as_secs()),
                None => "No data yet".to_owned(),
            };
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(footer).italics().small());
            });
        });
    }
}

// ---------------------------------------------------------------------------
// MQTT background thread
// ---------------------------------------------------------------------------

fn start_mqtt_thread(shared: Arc<Mutex<SensorData>>) {
    thread::spawn(move || {
        let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("MQTT_PORT")
            .unwrap_or_else(|_| "1883".to_string())
            .parse()
            .expect("MQTT_PORT must be a valid u16");

        let mut opts = MqttOptions::new("mqtt-dashboard", &host, port);
        opts.set_keep_alive(Duration::from_secs(5));

        let (client, mut connection) = Client::new(opts, 16);

        {
            shared.lock().unwrap().status = format!("Connecting to {host}:{port} …");
        }

        // Queue the subscription – it will be sent once the ConnAck arrives.
        if let Err(e) = client.subscribe("home/room/sensor", QoS::AtMostOnce) {
            shared.lock().unwrap().status = format!("Subscribe error: {e}");
            return;
        }

        for event in connection.iter() {
            match event {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    shared.lock().unwrap().status = "Connected".to_owned();
                }

                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let payload_str = match std::str::from_utf8(&publish.payload) {
                        Ok(s) => s,
                        Err(_) => {
                            shared.lock().unwrap().status =
                                "Error: payload is not valid UTF-8".to_owned();
                            continue;
                        }
                    };

                    match serde_json::from_str::<SensorReading>(payload_str) {
                        Ok(reading) => {
                            let mut data = shared.lock().unwrap();
                            data.temperature_c = Some(reading.temperature_c);
                            data.humidity_pct = Some(reading.humidity_pct);
                            data.last_update = Some(Instant::now());
                            data.status = "Connected".to_owned();
                        }
                        Err(e) => {
                            shared.lock().unwrap().status = format!("JSON parse error: {e}");
                        }
                    }
                }

                Err(e) => {
                    shared.lock().unwrap().status = format!("Connection error: {e}");
                    // Avoid a tight error-spin; the iterator will retry internally.
                    thread::sleep(Duration::from_secs(2));
                }

                // Ignore all other events (PingResp, SubAck, outgoing acks, …).
                _ => {}
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> eframe::Result<()> {
    let shared = Arc::new(Mutex::new(SensorData {
        status: "Waiting for data…".to_owned(),
        ..Default::default()
    }));

    start_mqtt_thread(Arc::clone(&shared));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MQTT Dashboard")
            .with_inner_size([480.0, 320.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "MQTT Dashboard",
        native_options,
        Box::new(move |_cc| Ok(Box::new(DashboardApp::new(shared)))),
    )
}
