mod types;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, Stroke};
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use types::{DeviceData, Qty};

// ── MQTT settings ────────────────────────────────────────────────────────────
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

// ── Scale maxima for progress bars ───────────────────────────────────────────
const MAX_CURRENT_A: f64 = 16.0;
const MAX_POWER_W: f64 = 7_500.0;

// ── Colour palette ───────────────────────────────────────────────────────────
const COL_BG: Color32 = Color32::from_rgb(18, 22, 36);
const COL_CARD: Color32 = Color32::from_rgb(28, 33, 50);
const COL_CARD_BORDER: Color32 = Color32::from_rgb(55, 62, 88);
const COL_LABEL: Color32 = Color32::from_rgb(140, 150, 180);
const COL_ACCENT_BLUE: Color32 = Color32::from_rgb(90, 190, 255);

const COL_GOOD: Color32 = Color32::from_rgb(50, 210, 120);
const COL_WARN: Color32 = Color32::from_rgb(255, 185, 50);
const COL_CRIT: Color32 = Color32::from_rgb(230, 60, 60);
const COL_CONNECTED: Color32 = Color32::from_rgb(50, 210, 120);
const COL_DISCONNECTED: Color32 = Color32::from_rgb(200, 60, 60);

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct AppState {
    data: Option<DeviceData>,
    connected: bool,
    last_update: Option<Instant>,
    last_error: Option<String>,
    host: String,
    port: u16,
    topic: String,
}

struct DashboardApp {
    state: Arc<Mutex<AppState>>,
}

impl DashboardApp {
    fn new(cc: &eframe::CreationContext<'_>, state: Arc<Mutex<AppState>>) -> Self {
        // Custom dark visuals with a slightly adjusted colour scheme
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = COL_BG;
        visuals.window_fill = COL_BG;
        cc.egui_ctx.set_visuals(visuals);

        // Slightly larger default font sizes
        let mut style = (*cc.egui_ctx.style()).clone();
        if let Some(s) = style.text_styles.get_mut(&egui::TextStyle::Body) {
            s.size = 13.5;
        };
        cc.egui_ctx.set_style(style);

        Self { state }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Interpolate a load fraction → green / amber / red.
fn load_color(fraction: f32) -> Color32 {
    if fraction < 0.5 {
        COL_GOOD
    } else if fraction < 0.8 {
        COL_WARN
    } else {
        COL_CRIT
    }
}

/// A single labelled row: label | value string right-aligned, then a progress bar.
fn metric_row(ui: &mut egui::Ui, icon: &str, label: &str, qty: &Qty, max: f64) {
    let fraction = (qty.value / max).clamp(0.0, 1.0) as f32;
    let color = load_color(fraction);
    let value_text = format!("{:.2} {}", qty.value, qty.unit);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{icon} {label}"))
                .color(COL_LABEL)
                .size(12.5),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(&value_text).color(color).strong().size(14.5));
        });
    });

    let _ = ui.add(
        egui::ProgressBar::new(fraction)
            .desired_width(ui.available_width())
            .fill(color),
    );
    ui.add_space(6.0);
}

/// Draw one phase card inside the provided `Ui`.
fn phase_card(ui: &mut egui::Ui, title: &str, current: &Qty, power: &Qty) {
    let frame = egui::Frame::default()
        .fill(COL_CARD)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(16))
        .stroke(Stroke::new(1.0, COL_CARD_BORDER));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());

        // Header row
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚡").size(18.0));
            ui.label(
                RichText::new(title)
                    .size(16.0)
                    .strong()
                    .color(COL_ACCENT_BLUE),
            );
        });

        ui.add(egui::Separator::default().spacing(10.0));
        ui.add_space(2.0);

        metric_row(ui, "〜", "Current", current, MAX_CURRENT_A);
        metric_row(ui, "🔋", "Power", power, MAX_POWER_W);
    });
}

/// Full-width info card with a big value on the right.
fn info_card(ui: &mut egui::Ui, icon: &str, label: &str, value: &str, value_color: Color32) {
    let frame = egui::Frame::default()
        .fill(COL_CARD)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(20, 14))
        .stroke(Stroke::new(1.0, COL_CARD_BORDER));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{icon}  {label}"))
                    .size(13.5)
                    .color(COL_LABEL),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(value).size(28.0).strong().color(value_color));
            });
        });
    });
}

// ── eframe App ────────────────────────────────────────────────────────────────

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh every 500 ms so the "last update" timer stays accurate.
        ctx.request_repaint_after(Duration::from_millis(500));

        let state = self.state.lock().unwrap();

        // ── Top panel ─────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::default()
                    .fill(COL_CARD)
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .stroke(Stroke::new(1.0, COL_CARD_BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("⚡  BE-Strom Dashboard")
                            .size(20.0)
                            .strong()
                            .color(COL_ACCENT_BLUE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection badge
                        let (dot_color, status_text) = if state.connected {
                            (COL_CONNECTED, "● Connected")
                        } else {
                            (COL_DISCONNECTED, "● Disconnected")
                        };
                        ui.label(RichText::new(status_text).color(dot_color).size(13.0));

                        ui.add_space(12.0);

                        // Last-update timestamp
                        if let Some(instant) = state.last_update {
                            let secs = instant.elapsed().as_secs();
                            let ts = if secs == 0 {
                                "Updated just now".to_owned()
                            } else {
                                format!("Updated {secs}s ago")
                            };
                            ui.label(RichText::new(ts).color(COL_LABEL).size(12.0));
                            ui.add_space(8.0);
                        }

                        // Device name
                        if let Some(data) = &state.data
                            && !data.device.is_empty()
                        {
                            ui.label(
                                RichText::new(format!("🖥  {}", data.device))
                                    .color(COL_LABEL)
                                    .size(12.5),
                            );
                        }
                    });
                });
            });

        // ── Central panel ──────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(COL_BG)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                if let Some(data) = &state.data {
                    // ── Voltage ───────────────────────────────────────────────
                    let v_text = format!("{:.1} {}", data.voltage.value, data.voltage.unit);
                    info_card(ui, "🔌", "Mains Voltage", &v_text, COL_ACCENT_BLUE);

                    ui.add_space(14.0);

                    // ── Three phase cards in equal columns ────────────────────
                    let phases = [
                        ("Phase 1", &data.current_1, &data.power_1),
                        ("Phase 2", &data.current_2, &data.power_2),
                        ("Phase 3", &data.current_3, &data.power_3),
                    ];

                    ui.columns(3, |cols| {
                        for (col, (title, current, power)) in cols.iter_mut().zip(phases.iter()) {
                            phase_card(col, title, current, power);
                        }
                    });

                    ui.add_space(14.0);

                    // ── Total power ───────────────────────────────────────────
                    let total_w = data.power_1.value + data.power_2.value + data.power_3.value;
                    let unit = &data.power_1.unit;
                    let total_text = format!("{total_w:.2} {unit}");
                    let total_color =
                        load_color((total_w / (MAX_POWER_W * 3.0)).clamp(0.0, 1.0) as f32);
                    info_card(
                        ui,
                        "⚡",
                        "Total Power (all phases)",
                        &total_text,
                        total_color,
                    );

                    // ── Mini power balance bar ────────────────────────────────
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        let bar_color = [
                            load_color((data.power_1.value / MAX_POWER_W).clamp(0.0, 1.0) as f32),
                            load_color((data.power_2.value / MAX_POWER_W).clamp(0.0, 1.0) as f32),
                            load_color((data.power_3.value / MAX_POWER_W).clamp(0.0, 1.0) as f32),
                        ];
                        let powers = [data.power_1.value, data.power_2.value, data.power_3.value];
                        let labels = ["L1", "L2", "L3"];
                        let total = powers.iter().sum::<f64>().max(1.0);

                        // Pre-compute once so all three bars get the same width.
                        let spacing = ui.spacing().item_spacing.x;
                        let bar_width = (ui.available_width() - spacing * 2.0) / 3.0;

                        for i in 0..3 {
                            let frac = (powers[i] / total) as f32;
                            ui.add(
                                egui::ProgressBar::new(frac)
                                    .desired_width(bar_width)
                                    .fill(bar_color[i])
                                    .text(
                                        RichText::new(format!(
                                            "{} {:.0}%",
                                            labels[i],
                                            frac * 100.0
                                        ))
                                        .size(11.0),
                                    ),
                            );
                        }
                    });

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Load balance across phases")
                            .color(COL_LABEL)
                            .size(11.0),
                    );
                } else {
                    // ── Waiting / error state ─────────────────────────────────
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(60.0);

                            if state.connected {
                                ui.label(
                                    RichText::new("⏳  Waiting for data…")
                                        .size(22.0)
                                        .color(COL_LABEL),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("Subscribed to '{}'", state.topic))
                                        .size(13.0)
                                        .color(Color32::from_rgb(80, 90, 120)),
                                );
                            } else {
                                ui.label(
                                    RichText::new("🔴  Not connected")
                                        .size(22.0)
                                        .color(COL_DISCONNECTED),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!(
                                        "Connecting to mqtt://{}:{} …",
                                        state.host, state.port
                                    ))
                                    .size(13.0)
                                    .color(Color32::from_rgb(80, 90, 120)),
                                );

                                if let Some(err) = &state.last_error {
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(format!("⚠  {err}"))
                                            .size(12.0)
                                            .color(COL_WARN),
                                    );
                                }
                            }
                        });
                    });
                }
            });
    }
}

// ── MQTT background thread ────────────────────────────────────────────────────

fn start_mqtt_thread(state: Arc<Mutex<AppState>>) {
    thread::Builder::new()
        .name("mqtt".into())
        .spawn(move || {
            loop {
                let s = state.lock().unwrap();
                let mut opts = MqttOptions::new("be-strom-dashboard", s.host.clone(), s.port);
                let topic = s.topic.clone();
                drop(s);

                opts.set_keep_alive(Duration::from_secs(10));
                opts.set_clean_session(true);

                let (client, mut connection) = Client::new(opts, 128);

                // Queue the subscription before driving the event loop.
                if let Err(e) = client.subscribe(topic, QoS::AtMostOnce) {
                    let mut s = state.lock().unwrap();
                    s.last_error = Some(format!("subscribe: {e}"));
                    drop(s);
                    thread::sleep(RECONNECT_DELAY);
                    continue;
                }

                for notification in connection.iter() {
                    match notification {
                        Ok(Event::Incoming(Packet::ConnAck(_))) => {
                            let mut s = state.lock().unwrap();
                            println!("[mqtt] connected to {}:{}", s.host, s.port);
                            s.connected = true;
                            s.last_error = None;
                        }

                        Ok(Event::Incoming(Packet::Publish(msg))) => {
                            match std::str::from_utf8(&msg.payload) {
                                Ok(text) => match serde_json::from_str::<DeviceData>(text) {
                                    Ok(data) => {
                                        let mut s = state.lock().unwrap();
                                        s.data = Some(data);
                                        s.last_update = Some(Instant::now());
                                    }
                                    Err(e) => {
                                        eprintln!("[mqtt] JSON parse error: {e}");
                                    }
                                },
                                Err(e) => {
                                    eprintln!("[mqtt] UTF-8 error: {e}");
                                }
                            }
                        }

                        Err(e) => {
                            eprintln!("[mqtt] connection error: {e:?}");
                            let mut s = state.lock().unwrap();
                            s.connected = false;
                            s.last_error = Some(format!("{e}"));
                            break;
                        }

                        _ => {} // ping/ack/outgoing — ignore
                    }
                }

                state.lock().unwrap().connected = false;
                eprintln!(
                    "[mqtt] disconnected — retrying in {}s…",
                    RECONNECT_DELAY.as_secs()
                );
                thread::sleep(RECONNECT_DELAY);
            }
        })
        .expect("failed to spawn mqtt thread");
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> eframe::Result<()> {
    let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1883);
    let topic = std::env::var("MQTT_TOPIC").unwrap_or_else(|_| "be-strom/data".to_string());

    let app_state = AppState {
        host,
        port,
        topic,
        ..Default::default()
    };
    let state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(app_state));

    start_mqtt_thread(Arc::clone(&state));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BE-Strom Dashboard")
            .with_inner_size([780.0, 480.0])
            .with_min_inner_size([640.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "BE-Strom Dashboard",
        native_options,
        Box::new(|cc| Ok(Box::new(DashboardApp::new(cc, Arc::clone(&state))))),
    )
}
