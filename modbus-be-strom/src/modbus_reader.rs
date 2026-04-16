use serde::Serialize;
use tokio_modbus::client::{Client, Reader, tcp};

#[derive(Debug, Serialize)]
pub struct DeviceData {
    pub device: String,
    pub voltage: f64,
    pub current_1: f64,
    pub current_2: f64,
    pub current_3: f64,
    pub power_1: f64,
    pub power_2: f64,
    pub power_3: f64,
}

// Common divisor for Voltage and Power
const DIVISOR_V_P: f64 = 92.16;
// Approximate divisor for Current (range 1.0 A)
const DIVISOR_I: f64 = 27648.0;

/// Reads the current values from the device at the given address.
///
pub async fn read_current(address: &str) -> anyhow::Result<DeviceData> {
    let socket_addr = address.parse().unwrap();
    let mut ctx = tcp::connect(socket_addr).await?;

    println!("Fetching the device ID ...");
    let data = ctx.read_input_registers(0x1000, 7).await??;
    let bytes: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
        x.push((elem & 0xff) as u8);
        x.push((elem >> 8) as u8);
        x
    });
    let device = String::from_utf8(bytes).unwrap();

    println!("Fetching the current values ...");
    let raw_data = ctx.read_input_registers(0x0000, 7).await??;
    let voltage = f64::from(raw_data[0]) / DIVISOR_V_P;
    let current_1 = f64::from(raw_data[1]) / DIVISOR_I;
    let current_2 = f64::from(raw_data[2]) / DIVISOR_I;
    let current_3 = f64::from(raw_data[3]) / DIVISOR_I;
    let power_1 = f64::from(raw_data[4]) / DIVISOR_V_P;
    let power_2 = f64::from(raw_data[5]) / DIVISOR_V_P;
    let power_3 = f64::from(raw_data[6]) / DIVISOR_V_P;

    let be_strom = DeviceData {
        device,
        voltage,
        current_1,
        current_2,
        current_3,
        power_1,
        power_2,
        power_3,
    };

    println!("Disconnecting");
    ctx.disconnect().await?;

    Ok(be_strom)
}
