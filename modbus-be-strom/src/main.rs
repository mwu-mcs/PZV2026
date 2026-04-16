//! Modbus TCP client for Weidmüller UR20-3EM-230V-AC energy measurement module
//!
//! Hardware:
//!   Fieldbus coupler : UR20-FBC-MOD-TCP-V2  @ 10.234.1.82:502
//!   Energy module    : UR20-3EM-230V-AC      (slot 1, base address 0)
//!
//! The UR20-3EM-230V-AC delivers 20 × IEEE 754 float32 values (40 registers /
//! 80 bytes) via Function Code 3 (Read Holding Registers).  Each float32 is
//! stored in two consecutive 16-bit registers, high word first (big-endian).
//!
//! Register map (0-based, per channel then shared):
//!   CH1: 0-1 U [V], 2-3 I [A], 4-5 P [W], 6-7 Q [VAr], 8-9 S [VA], 10-11 PF
//!   CH2: 12-13 U,   14-15 I,   16-17 P,   18-19 Q,      20-21 S,    22-23 PF
//!   CH3: 24-25 U,   26-27 I,   28-29 P,   30-31 Q,      32-33 S,    34-35 PF
//!   Shared: 36-37 f [Hz],  38-39 ΣE [kWh]
//!
//! NOTE: If the values look wrong (e.g. NaN / garbage) the register addresses
//! or the float byte-order may differ on your firmware version.  Run with
//! `--raw` to inspect the raw register hex values and adjust the constants.

use std::{net::SocketAddr, time::Duration};

use tokio::time::sleep;
use tokio_modbus::{prelude::Client,
    client::{tcp, Reader},
    slave::Slave,
};

// ── Connection settings ──────────────────────────────────────────────────────

const DEVICE_ADDR: &str = "10.234.1.82:502";

/// Modbus unit ID.
/// The FBC-MOD-TCP-V2 uses 0xFF (255) for direct TCP connections.
const UNIT_ID: u8 = 0xFF;

// ── Module register layout ───────────────────────────────────────────────────

/// Starting register address of the UR20-3EM-230V-AC in the process image.
/// Stays 0 as long as it is the first (or only) module on the coupler.
const BASE_ADDR: u16 = 0x0000;

/// Total number of 16-bit registers to read (20 float32 × 2 = 40).
const REG_COUNT: u16 = 40;

// Byte-offsets (in 16-bit words) for each measurement within the block:
//                             word offset
const OFF_U_CH1: usize = 0;  // Voltage     Channel 1  [V]
const OFF_I_CH1: usize = 2;  // Current     Channel 1  [A]
const OFF_P_CH1: usize = 4;  // ActivePwr   Channel 1  [W]
const OFF_Q_CH1: usize = 6;  // ReactivePwr Channel 1  [VAr]
const OFF_S_CH1: usize = 8;  // ApparentPwr Channel 1  [VA]
const OFF_PF_CH1: usize = 10; // PowerFactor Channel 1

const OFF_U_CH2: usize = 12;
const OFF_I_CH2: usize = 14;
const OFF_P_CH2: usize = 16;
const OFF_Q_CH2: usize = 18;
const OFF_S_CH2: usize = 20;
const OFF_PF_CH2: usize = 22;

const OFF_U_CH3: usize = 24;
const OFF_I_CH3: usize = 26;
const OFF_P_CH3: usize = 28;
const OFF_Q_CH3: usize = 30;
const OFF_S_CH3: usize = 32;
const OFF_PF_CH3: usize = 34;

const OFF_FREQ: usize = 36;   // Frequency  [Hz]
const OFF_ETOTAL: usize = 38; // Total active energy [kWh]

// ── Data types ───────────────────────────────────────────────────────────────

/// All measurements for a single channel of the UR20-3EM-230V-AC.
#[derive(Debug)]
struct Channel {
    label: &'static str,
    voltage: f32,        // V
    current: f32,        // A
    active_power: f32,   // W
    reactive_power: f32, // VAr
    apparent_power: f32, // VA
    power_factor: f32,   // dimensionless
}

/// All measurements from the UR20-3EM-230V-AC module.
#[derive(Debug)]
struct EnergyMeasurements {
    channels: [Channel; 3],
    frequency: f32,         // Hz
    total_energy_kwh: f32,  // kWh  (sum of all channels)
}

impl EnergyMeasurements {
    fn total_active_power(&self) -> f32 {
        self.channels.iter().map(|c| c.active_power).sum()
    }
    fn total_reactive_power(&self) -> f32 {
        self.channels.iter().map(|c| c.reactive_power).sum()
    }
    fn total_apparent_power(&self) -> f32 {
        self.channels.iter().map(|c| c.apparent_power).sum()
    }
}

// ── Register parsing ─────────────────────────────────────────────────────────

/// Decodes two consecutive Modbus registers (high word first) into an f32.
#[inline]
fn to_f32(regs: &[u16], word_offset: usize) -> f32 {
    let hi = regs[word_offset] as u32;
    let lo = regs[word_offset + 1] as u32;
    f32::from_bits((hi << 16) | lo)
}

fn parse_measurements(regs: &[u16]) -> EnergyMeasurements {
    let ch = |u_off, i_off, p_off, q_off, s_off, pf_off, label| Channel {
        label,
        voltage: to_f32(regs, u_off),
        current: to_f32(regs, i_off),
        active_power: to_f32(regs, p_off),
        reactive_power: to_f32(regs, q_off),
        apparent_power: to_f32(regs, s_off),
        power_factor: to_f32(regs, pf_off),
    };

    EnergyMeasurements {
        channels: [
            ch(OFF_U_CH1, OFF_I_CH1, OFF_P_CH1, OFF_Q_CH1, OFF_S_CH1, OFF_PF_CH1, "L1"),
            ch(OFF_U_CH2, OFF_I_CH2, OFF_P_CH2, OFF_Q_CH2, OFF_S_CH2, OFF_PF_CH2, "L2"),
            ch(OFF_U_CH3, OFF_I_CH3, OFF_P_CH3, OFF_Q_CH3, OFF_S_CH3, OFF_PF_CH3, "L3"),
        ],
        frequency: to_f32(regs, OFF_FREQ),
        total_energy_kwh: to_f32(regs, OFF_ETOTAL),
    }
}

// ── Printing ─────────────────────────────────────────────────────────────────

fn print_measurements(m: &EnergyMeasurements) {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  UR20-3EM-230V-AC  –  Energy Measurements           │");
    println!("├───────────┬──────────┬──────────┬────────────────────┤");
    println!("│ Channel   │    Voltage │   Current │   Active Power   │");
    println!("├───────────┼────────────┼───────────┼──────────────────┤");
    for ch in &m.channels {
        println!(
            "│  {:<8} │ {:>8.2} V │ {:>7.3} A │ {:>12.2} W   │",
            ch.label, ch.voltage, ch.current, ch.active_power
        );
    }
    println!("├───────────┴────────────┴───────────┴──────────────────┤");
    println!(
        "│  Total active power   : {:>10.2} W                  │",
        m.total_active_power()
    );
    println!(
        "│  Total reactive power : {:>10.2} VAr               │",
        m.total_reactive_power()
    );
    println!(
        "│  Total apparent power : {:>10.2} VA                │",
        m.total_apparent_power()
    );
    println!("├──────────────────────────────────────────────────────┤");
    println!("│  Per-channel details                                  │");
    println!("├──────────────────────────────────────────────────────┤");
    for ch in &m.channels {
        println!(
            "│  {} │ Q={:>8.2} VAr │ S={:>8.2} VA │ PF={:>6.4}  │",
            ch.label, ch.reactive_power, ch.apparent_power, ch.power_factor
        );
    }
    println!("├──────────────────────────────────────────────────────┤");
    println!("│  Frequency          : {:>8.3} Hz                    │", m.frequency);
    println!("│  Total active energy: {:>10.3} kWh                 │", m.total_energy_kwh);
    println!("└──────────────────────────────────────────────────────┘");
}

/// Dumps every register value as a hex / decimal pair for debugging.
fn print_raw(regs: &[u16]) {
    println!("Raw register dump ({} registers @ address {:#06x}):", regs.len(), BASE_ADDR);
    println!("{:>5}  {:>6}  {:>7}   decoded-float32", "Reg#", "0xHEX", "uint16");
    for (i, chunk) in regs.chunks(2).enumerate() {
        if chunk.len() == 2 {
            let val = to_f32(regs, i * 2);
            println!(
                "{:>4}+{:<1}  0x{:04X}  {:>7}  ┐",
                i * 2, 0, chunk[0], chunk[0]
            );
            println!(
                "{:>4}+{:<1}  0x{:04X}  {:>7}  ┘ → {:.6}",
                i * 2, 1, chunk[1], chunk[1], val
            );
        }
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_mode = std::env::args().any(|a| a == "--raw");
    let once = std::env::args().any(|a| a == "--once");
    let poll_secs: u64 = std::env::args()
        .skip_while(|a| a != "--interval")
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let socket_addr: SocketAddr = DEVICE_ADDR.parse()?;

    println!("Connecting to {} (unit id 0x{:02X}) …", DEVICE_ADDR, UNIT_ID);

    let mut ctx = tcp::connect_slave(socket_addr, Slave(UNIT_ID)).await?;

    println!("Connected. Reading UR20-3EM-230V-AC starting at register {BASE_ADDR}.");
    if !once {
        println!("Polling every {poll_secs}s.  Press Ctrl-C to stop.");
    }

    loop {
        match ctx.read_holding_registers(BASE_ADDR, REG_COUNT).await {
            Ok(Ok(regs)) => {
                if raw_mode {
                    print_raw(&regs);
                } else {
                    let m = parse_measurements(&regs);
                    print_measurements(&m);
                }
            }
            Ok(Err(exc)) => {
                eprintln!("Modbus exception from device: {exc}");
            }
            Err(err) => {
                eprintln!("Transport error: {err}");
                // Try to reconnect once before exiting
                eprintln!("Attempting reconnect …");
                match tcp::connect_slave(socket_addr, Slave(UNIT_ID)).await {
                    Ok(new_ctx) => {
                        ctx = new_ctx;
                        eprintln!("Reconnected.");
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }

        if once {
            break;
        }
        sleep(Duration::from_secs(poll_secs)).await;
    }

    ctx.disconnect().await?;
    Ok(())
}
