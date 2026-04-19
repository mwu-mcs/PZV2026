mod modbus_reader;
mod mqtt_pub;
mod types;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    mqtt_pub::run().await
}
