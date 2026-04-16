use serde::Serialize;

#[derive(Debug, Serialize)]
struct DeviceData {
    device: String,
    voltage: f64,
    current_1: f64,
    current_2: f64,
    current_3: f64,
    power_1: f64,
    power_2: f64,
    power_3: f64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tokio_modbus::prelude::*;

    let socket_addr = "10.234.1.82:502".parse().unwrap();

    let mut ctx = tcp::connect(socket_addr).await?;

    println!("Fetching the coupler ID");
    let data = ctx.read_input_registers(0x1000, 7).await??;

    let bytes: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
        x.push((elem & 0xff) as u8);
        x.push((elem >> 8) as u8);
        x
    });
    let device = String::from_utf8(bytes).unwrap();
    println!("The coupler ID is '{device}'");

    let raw_data = ctx.read_input_registers(0x0000, 7).await??;
    // Common divisor for Voltage and Power
    const DIVISOR_V_P: f64 = 92.16;
    // Approximate divisor for Current (range 1.0 A)
    const DIVISOR_I: f64 = 27648.0;

    let voltage = f64::from(raw_data[0]) / DIVISOR_V_P;
    let current_1 = f64::from(raw_data[1]) / DIVISOR_I;
    let current_2 = f64::from(raw_data[2]) / DIVISOR_I;
    let current_3 = f64::from(raw_data[3]) / DIVISOR_I;
    let power_1 = f64::from(raw_data[4]) / DIVISOR_V_P;
    let power_2 = f64::from(raw_data[5]) / DIVISOR_V_P;
    let power_3 = f64::from(raw_data[6]) / DIVISOR_V_P;

    println!("Voltage:   {voltage:.3} V");
    println!("Current 1: {current_1:.3} A");
    println!("Current 2: {current_2:.3} A");
    println!("Current 3: {current_3:.3} A");
    println!("Power 1:   {power_1:.3} W");
    println!("Power 2:   {power_2:.3} W");
    println!("Power 3:   {power_3:.3} W");

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

    println!("BeStrom: {be_strom:#?}");

    println!("Disconnecting");
    ctx.disconnect().await?;

    Ok(())
}
