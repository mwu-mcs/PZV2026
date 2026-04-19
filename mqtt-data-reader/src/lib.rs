#[cxx::bridge]
mod ffi {

    unsafe extern "C++" {
        include!("IDataSource.h");
        include!("DataPoint.h");
        include!("ChannelStats.h");

        #[namespace = "common"]
        type IDataSource;
        #[namespace = "common"]
        type DataPoint;
        #[namespace = "common"]
        type ChannelStats;

        fn open(self: Pin<&mut MqttReader>);
        fn close(self: Pin<&mut MqttReader>);
        fn is_open(self: &MqttReader) -> bool;
        fn read_all(self: Pin<&mut MqttReader>) -> bool;
        fn get(self: Pin<&mut MqttReader>, name: &CxxString, value: Pin<&mut DataPoint>) -> bool;
        fn get_id(self: &MqttReader) -> String;
        fn get_stats(self: &MqttReader) -> &ChannelStats;
    }

    pub struct MqttReader {
        name: String,
    }

    extern "Rust" {
        fn create_mqtt_reader(config: &CxxString) -> UniquePtr<IDataSource>;
    }
}

fn create_mqtt_reader(_config: &cxx::CxxString) -> cxx::UniquePtr<ffi::IDataSource> {
    cxx::UniquePtr::null()
}
