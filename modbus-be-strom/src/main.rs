use crate::modbus_reader::read_current;

mod modbus_reader;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let data = read_current("10.234.1.82:502").await?;

    println!("\nDevice: {}\n", data.device);
    println!("Voltage:   {}", data.voltage);
    println!("Current 1: {}", data.current_1);
    println!("Current 2: {}", data.current_2);
    println!("Current 3: {}", data.current_3);
    println!("Power 1:   {}", data.power_1);
    println!("Power 2:   {}", data.power_2);
    println!("Power 3:   {}", data.power_3);

    Ok(())
}
