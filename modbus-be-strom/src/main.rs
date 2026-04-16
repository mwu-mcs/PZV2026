use crate::modbus_reader::read_current;

mod modbus_reader;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let data = read_current("10.234.1.82:502").await?;

    println!("\nDevice:    {}\n", data.device);
    println!("Voltage:   {:.3} V", data.voltage);
    println!("Current 1: {:.3} A", data.current_1);
    println!("Current 2: {:.3} A", data.current_2);
    println!("Current 3: {:.3} A", data.current_3);
    println!("Power 1:   {:.3} W", data.power_1);
    println!("Power 2:   {:.3} W", data.power_2);
    println!("Power 3:   {:.3} W", data.power_3);

    Ok(())
}
