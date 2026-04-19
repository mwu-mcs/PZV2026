#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("DataPoint.h");
        include!("ChannelStats.h");

        #[namespace = "common"]
        type DataPoint;
        #[namespace = "common"]
        type ChannelStats;

        // ChannelStats state-transition helpers (inline functions from ChannelStats.h)
        #[namespace = "common"]
        fn connecting(stats: Pin<&mut ChannelStats>);
        #[namespace = "common"]
        fn connected(stats: Pin<&mut ChannelStats>);
        #[namespace = "common"]
        fn disconnected(stats: Pin<&mut ChannelStats>);
        #[namespace = "common"]
        #[rust_name = "successful_read"]
        fn successfulRead(stats: Pin<&mut ChannelStats>, read_duration_ms: f64);
    }

    extern "Rust" {
        type MqttReader;

        fn new_mqtt_reader(config: &CxxString) -> Box<MqttReader>;

        fn open(self: &mut MqttReader, stats: Pin<&mut ChannelStats>);
        fn close(self: &mut MqttReader, stats: Pin<&mut ChannelStats>);
        fn is_open(self: &MqttReader) -> bool;
        fn read_all(self: &mut MqttReader, stats: Pin<&mut ChannelStats>) -> bool;
        fn get(self: &mut MqttReader, name: &CxxString, value: Pin<&mut DataPoint>) -> bool;
        fn get_id(self: &MqttReader) -> String;
    }
}

/// The Rust implementation of an MQTT data source.
pub struct MqttReader {
    id: String,
    open: bool,
    // TODO: MQTT client handle, topic subscriptions, value cache, ...
}

pub fn new_mqtt_reader(config: &cxx::CxxString) -> Box<MqttReader> {
    Box::new(MqttReader {
        id: config.to_string(),
        open: false,
    })
}

impl MqttReader {
    pub fn open(&mut self, mut stats: std::pin::Pin<&mut ffi::ChannelStats>) {
        ffi::connecting(stats.as_mut());
        // TODO: parse self.id as a connection config (e.g. JSON) and establish
        //       the MQTT broker connection.
        // On success:
        //   self.open = true;
        //   ffi::connected(stats);
        // On failure:
        //   // ffi::cannotConnect(stats, &error_msg, cause);
        todo!("connect to MQTT broker")
    }

    pub fn close(&mut self, stats: std::pin::Pin<&mut ffi::ChannelStats>) {
        // TODO: gracefully disconnect from the MQTT broker.
        self.open = false;
        ffi::disconnected(stats);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn read_all(&mut self, mut stats: std::pin::Pin<&mut ffi::ChannelStats>) -> bool {
        if !self.open {
            return false;
        }
        // TODO: poll subscribed MQTT topics and update the internal value cache.
        // On success:
        //   ffi::successful_read(stats.as_mut(), duration_ms);
        //   return true;
        // On failure:
        //   // ffi::failed_read(stats, &error_msg, cause);
        //   return false;
        todo!("poll MQTT topics")
    }

    pub fn get(
        &mut self,
        _name: &cxx::CxxString,
        _value: std::pin::Pin<&mut ffi::DataPoint>,
    ) -> bool {
        // TODO: look up _name in the internal value cache and write the result
        //       into _value via DataPoint setter methods.
        false
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
