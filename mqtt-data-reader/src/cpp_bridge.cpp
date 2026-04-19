// C++ bridge: wraps the Rust MqttReader inside a common::IDataSource concrete class
// and exports the `createMqttDataSource` factory symbol expected by ReaderFactory.

#include "IDataSource.h"
#include "ChannelStats.h"
#include "mqtt-data-reader/src/lib.rs.h"

#include <memory>
#include <string>

namespace {

/// Adapts a Rust-owned MqttReader to the common::IDataSource interface.
/// ChannelStats are owned here so getStats() can return a stable reference.
class MqttDataSourceBridge final : public common::IDataSource
{
public:
    explicit MqttDataSourceBridge(rust::Box<MqttReader> reader)
        : m_reader(std::move(reader)), m_stats{}
    {}

    void open()   override { m_reader->open(m_stats);  }
    void close()  override { m_reader->close(m_stats); }

    bool isOpen()   const override { return m_reader->is_open(); }
    bool readAll()        override { return m_reader->read_all(m_stats); }

    bool get(const std::string& name, common::DataPoint& value) override
    {
        return m_reader->get(name, value);
    }

    std::string getId() const override
    {
        return static_cast<std::string>(m_reader->get_id());
    }

    const common::ChannelStats& getStats() const override { return m_stats; }

private:
    rust::Box<MqttReader> m_reader;
    common::ChannelStats  m_stats;
};

} // anonymous namespace

// TODO: update the signature to match CreateDataSourceFunction in ReaderFactory,
//       which typically is:
//         std::shared_ptr<common::IDataSource>(const common::config::ReaderConfig&)
//       extern "C" keeps the symbol name unmangled so dll::import_symbol can find
//       it by the plain string "createMqttDataSource".
extern "C" std::shared_ptr<common::IDataSource>
createMqttDataSource(const char* config_json)
{
    std::string config = config_json ? config_json : "";
    auto reader = new_mqtt_reader(config);
    return std::make_shared<MqttDataSourceBridge>(std::move(reader));
}
