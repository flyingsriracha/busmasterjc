/*
 * BUSMASTER Web - Core Types
 * 
 * Platform-independent type definitions extracted from legacy BUSMASTER
 * Removes Windows-specific dependencies (HWND, HRESULT, etc.)
 */

#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace BusmasterCore {

// Result codes (replacing HRESULT)
enum class Result {
    OK = 0,
    ERROR_INVALID_PARAM = 1,
    ERROR_NOT_CONNECTED = 2,
    ERROR_ALREADY_CONNECTED = 3,
    ERROR_DRIVER_NOT_FOUND = 4,
    ERROR_HARDWARE_NOT_FOUND = 5,
    ERROR_SEND_FAILED = 6,
    ERROR_RECEIVE_FAILED = 7,
    ERROR_UNKNOWN = 99
};

// Bus types
enum class BusType {
    CAN,
    LIN,
    FLEXRAY,
    J1939
};

// Driver types
enum class DriverType {
    VIRTUAL_CAN = 0,
    PEAK_USB = 1,
    VECTOR_XL = 2,
    ETAS_BOA = 3,
    KVASER = 4,
    ICS_NEOVI = 5,
    MHS = 6,
    IXXAT = 7
};

// Controller mode
enum class ControllerMode {
    ACTIVE = 1,
    PASSIVE = 2
};

// Message direction
enum class Direction {
    RX = 0,
    TX = 1
};

// Error states
enum class ErrorState {
    ERROR_ACTIVE = 0,
    ERROR_WARNING,
    ERROR_PASSIVE,
    ERROR_BUS_OFF,
    ERROR_FRAME
};

// CAN Message structure (platform-independent)
struct CANMessage {
    uint32_t id;                    // Message ID (11 or 29 bit)
    bool extended;                  // Extended frame (29-bit)
    bool rtr;                       // Remote transmission request
    bool canfd;                     // CAN FD frame
    uint8_t length;                 // Data length (0-64)
    uint8_t channel;                // Channel number
    uint8_t data[64];               // Data bytes
    uint64_t timestamp;             // Timestamp in microseconds
    Direction direction;            // TX or RX
};

// Controller configuration
struct ControllerConfig {
    std::string name;               // Controller name
    uint32_t baudrate;              // Baudrate (e.g., 500000 for 500kbps)
    uint32_t samplePoint;           // Sample point percentage
    ControllerMode mode;            // Active or Passive
    bool selfReception;             // Enable self-reception
    
    // CAN FD parameters
    bool canFDEnabled;              // Enable CAN FD
    uint32_t dataBitRate;           // Data phase bitrate
    uint32_t dataSamplePoint;       // Data phase sample point
};

// Hardware interface information
struct HardwareInterface {
    std::string id;                 // Unique identifier
    std::string name;               // Display name
    std::string description;        // Description
    DriverType type;                // Driver type
    bool available;                 // Is hardware available
    uint32_t channelCount;          // Number of channels
};

// Driver information
struct DriverInfo {
    std::string id;                 // Driver ID
    std::string name;               // Display name
    std::string version;            // Driver version
    DriverType type;                // Driver type
    bool available;                 // Is driver available
    std::vector<HardwareInterface> interfaces; // Available hardware
};

// Network statistics
struct NetworkStatistics {
    uint64_t totalMessages;         // Total messages
    uint32_t messagesPerSecond;     // Current msg/sec rate
    double busLoad;                 // Bus load percentage (0-100)
    uint32_t errorFrames;           // Total error frames
    uint32_t txErrors;              // TX error count
    uint32_t rxErrors;              // RX error count
    uint64_t timestamp;             // Timestamp
};

// Error counter
struct ErrorCounter {
    uint8_t txErrors;               // TX error count
    uint8_t rxErrors;               // RX error count
    ErrorState state;               // Controller error state
};

// Filter configuration
struct MessageFilter {
    uint32_t id;                    // Message ID
    uint32_t mask;                  // Acceptance mask
    bool extended;                  // Extended frame
    bool enabled;                   // Filter enabled
};

// Callback function types
using MessageCallback = void(*)(const CANMessage& msg, void* userData);
using ErrorCallback = void(*)(const std::string& error, void* userData);
using StatusCallback = void(*)(ErrorState state, void* userData);

} // namespace BusmasterCore

