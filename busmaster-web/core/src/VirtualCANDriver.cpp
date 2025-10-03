/*
 * BUSMASTER Web - Virtual CAN Driver Implementation
 */

#include "VirtualCANDriver.h"
#include <chrono>
#include <cstring>

namespace BusmasterCore {

VirtualCANDriver::VirtualCANDriver()
    : m_initialized(false)
    , m_connected(false)
    , m_running(false)
    , m_callback(nullptr)
    , m_callbackUserData(nullptr)
    , m_stopWorker(false)
{
    memset(&m_stats, 0, sizeof(m_stats));
    memset(&m_errorCounter, 0, sizeof(m_errorCounter));
    m_errorCounter.state = ErrorState::ERROR_ACTIVE;
}

VirtualCANDriver::~VirtualCANDriver() {
    Shutdown();
}

Result VirtualCANDriver::Initialize() {
    if (m_initialized) {
        return Result::OK;
    }

    m_initialized = true;
    m_lastError = "";
    return Result::OK;
}

Result VirtualCANDriver::Shutdown() {
    if (m_running) {
        Stop();
    }
    
    if (m_connected) {
        DeselectHardware();
    }
    
    m_initialized = false;
    return Result::OK;
}

Result VirtualCANDriver::GetDriverInfo(DriverInfo& info) {
    info.id = "virtual-can";
    info.name = "Virtual CAN";
    info.version = "1.0.0";
    info.type = DriverType::VIRTUAL_CAN;
    info.available = true;
    info.interfaces.clear();
    
    return Result::OK;
}

Result VirtualCANDriver::ListHardware(std::vector<HardwareInterface>& interfaces) {
    interfaces.clear();
    
    // Virtual CAN provides up to 4 virtual channels
    for (int i = 0; i < 4; i++) {
        HardwareInterface hwInterface;
        hwInterface.id = "virtual-can-" + std::to_string(i);
        hwInterface.name = "Virtual CAN Channel " + std::to_string(i + 1);
        hwInterface.description = "Software simulated CAN channel";
        hwInterface.type = DriverType::VIRTUAL_CAN;
        hwInterface.available = true;
        hwInterface.channelCount = 1;
        interfaces.push_back(hwInterface);
    }
    
    return Result::OK;
}

Result VirtualCANDriver::SelectHardware(const std::string& interfaceId, 
                                        const ControllerConfig& config) {
    if (!m_initialized) {
        m_lastError = "Driver not initialized";
        return Result::ERROR_NOT_CONNECTED;
    }
    
    m_config = config;
    m_connected = true;
    m_lastError = "";
    
    return Result::OK;
}

Result VirtualCANDriver::DeselectHardware() {
    if (m_running) {
        Stop();
    }
    
    m_connected = false;
    return Result::OK;
}

Result VirtualCANDriver::Start() {
    if (!m_connected) {
        m_lastError = "Hardware not selected";
        return Result::ERROR_NOT_CONNECTED;
    }
    
    if (m_running) {
        return Result::OK;
    }
    
    m_running = true;
    m_stopWorker = false;
    
    // Start worker thread for message processing
    m_workerThread = std::thread(&VirtualCANDriver::WorkerThread, this);
    
    return Result::OK;
}

Result VirtualCANDriver::Stop() {
    if (!m_running) {
        return Result::OK;
    }
    
    m_stopWorker = true;
    
    if (m_workerThread.joinable()) {
        m_workerThread.join();
    }
    
    m_running = false;
    
    // Clear queues
    std::lock_guard<std::mutex> lock(m_mutex);
    while (!m_txQueue.empty()) m_txQueue.pop();
    while (!m_rxQueue.empty()) m_rxQueue.pop();
    
    return Result::OK;
}

Result VirtualCANDriver::SendMessage(const CANMessage& msg) {
    if (!m_running) {
        m_lastError = "Controller not running";
        return Result::ERROR_NOT_CONNECTED;
    }
    
    // Add to TX queue
    {
        std::lock_guard<std::mutex> lock(m_mutex);
        m_txQueue.push(msg);
    }
    
    // In loopback mode, immediately echo back as RX
    if (m_config.selfReception) {
        CANMessage rxMsg = msg;
        rxMsg.direction = Direction::RX;
        rxMsg.timestamp = GetTimestamp();
        
        std::lock_guard<std::mutex> lock(m_mutex);
        m_rxQueue.push(rxMsg);
    }
    
    m_stats.totalMessages++;
    UpdateStatistics(msg);
    
    return Result::OK;
}

Result VirtualCANDriver::RegisterMessageCallback(MessageCallback callback, void* userData) {
    m_callback = callback;
    m_callbackUserData = userData;
    return Result::OK;
}

Result VirtualCANDriver::UnregisterMessageCallback() {
    m_callback = nullptr;
    m_callbackUserData = nullptr;
    return Result::OK;
}

Result VirtualCANDriver::GetErrorCounter(ErrorCounter& counter, uint8_t channel) {
    counter = m_errorCounter;
    return Result::OK;
}

Result VirtualCANDriver::GetStatistics(NetworkStatistics& stats) {
    stats = m_stats;
    stats.timestamp = GetTimestamp();
    return Result::OK;
}

Result VirtualCANDriver::SetFilters(const std::vector<MessageFilter>& filters) {
    std::lock_guard<std::mutex> lock(m_mutex);
    m_filters = filters;
    return Result::OK;
}

std::string VirtualCANDriver::GetLastError() const {
    return m_lastError;
}

bool VirtualCANDriver::IsConnected() const {
    return m_connected;
}

void VirtualCANDriver::WorkerThread() {
    while (!m_stopWorker) {
        // Process RX queue
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            
            while (!m_rxQueue.empty()) {
                CANMessage msg = m_rxQueue.front();
                m_rxQueue.pop();
                
                // Apply filters
                if (PassesFilter(msg)) {
                    // Call registered callback
                    if (m_callback) {
                        m_callback(msg, m_callbackUserData);
                    }
                }
            }
        }
        
        // Sleep for 1ms
        std::this_thread::sleep_for(std::chrono::milliseconds(1));
    }
}

bool VirtualCANDriver::PassesFilter(const CANMessage& msg) {
    if (m_filters.empty()) {
        return true; // No filters = accept all
    }
    
    for (const auto& filter : m_filters) {
        if (!filter.enabled) continue;
        
        if (filter.extended != msg.extended) continue;
        
        uint32_t maskedId = msg.id & filter.mask;
        uint32_t filterId = filter.id & filter.mask;
        
        if (maskedId == filterId) {
            return true;
        }
    }
    
    return false;
}

void VirtualCANDriver::UpdateStatistics(const CANMessage& msg) {
    // Update message count
    m_stats.totalMessages++;
    
    // Calculate bus load (simplified)
    // Assume standard CAN frame: ~130 bits at 500 kbps
    // This is a rough estimation
    double bitsPerMessage = 130.0;
    double timePerMessage = bitsPerMessage / m_config.baudrate;
    m_stats.busLoad = std::min(100.0, m_stats.busLoad + timePerMessage * 100.0);
    
    // Decay bus load over time
    if (m_stats.busLoad > 0) {
        m_stats.busLoad *= 0.99; // Decay factor
    }
}

uint64_t VirtualCANDriver::GetTimestamp() {
    auto now = std::chrono::system_clock::now();
    auto duration = now.time_since_epoch();
    return std::chrono::duration_cast<std::chrono::microseconds>(duration).count();
}

} // namespace BusmasterCore

