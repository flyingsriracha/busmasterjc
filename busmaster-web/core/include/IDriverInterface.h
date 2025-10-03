/*
 * BUSMASTER Web - Driver Interface Layer
 * 
 * Platform-independent interface for hardware drivers
 * Extracted and modernized from legacy BaseDIL_CAN
 */

#pragma once

#include "BusmasterTypes.h"
#include <vector>
#include <memory>

namespace BusmasterCore {

/**
 * @brief Base interface for all hardware drivers
 * 
 * This interface must be implemented by all hardware driver plugins.
 * It provides a platform-independent way to interact with CAN/LIN hardware.
 */
class IDriverInterface {
public:
    virtual ~IDriverInterface() = default;

    /**
     * @brief Initialize the driver
     * @return Result::OK on success
     */
    virtual Result Initialize() = 0;

    /**
     * @brief Cleanup and close the driver
     * @return Result::OK on success
     */
    virtual Result Shutdown() = 0;

    /**
     * @brief Get driver information
     * @param info Output parameter for driver information
     * @return Result::OK on success
     */
    virtual Result GetDriverInfo(DriverInfo& info) = 0;

    /**
     * @brief List available hardware interfaces
     * @param interfaces Output vector of available hardware
     * @return Result::OK on success
     */
    virtual Result ListHardware(std::vector<HardwareInterface>& interfaces) = 0;

    /**
     * @brief Select and configure hardware
     * @param interfaceId Hardware interface ID
     * @param config Controller configuration
     * @return Result::OK on success
     */
    virtual Result SelectHardware(const std::string& interfaceId, 
                                   const ControllerConfig& config) = 0;

    /**
     * @brief Deselect hardware and release resources
     * @return Result::OK on success
     */
    virtual Result DeselectHardware() = 0;

    /**
     * @brief Start the CAN controller
     * @return Result::OK on success
     */
    virtual Result Start() = 0;

    /**
     * @brief Stop the CAN controller
     * @return Result::OK on success
     */
    virtual Result Stop() = 0;

    /**
     * @brief Send a CAN message
     * @param msg Message to send
     * @return Result::OK on success
     */
    virtual Result SendMessage(const CANMessage& msg) = 0;

    /**
     * @brief Register callback for received messages
     * @param callback Function to call when message is received
     * @param userData User data to pass to callback
     * @return Result::OK on success
     */
    virtual Result RegisterMessageCallback(MessageCallback callback, 
                                           void* userData) = 0;

    /**
     * @brief Unregister message callback
     * @return Result::OK on success
     */
    virtual Result UnregisterMessageCallback() = 0;

    /**
     * @brief Get current error counters
     * @param counter Output parameter for error counters
     * @param channel Channel number
     * @return Result::OK on success
     */
    virtual Result GetErrorCounter(ErrorCounter& counter, uint8_t channel) = 0;

    /**
     * @brief Get network statistics
     * @param stats Output parameter for statistics
     * @return Result::OK on success
     */
    virtual Result GetStatistics(NetworkStatistics& stats) = 0;

    /**
     * @brief Configure message filters
     * @param filters Vector of filter configurations
     * @return Result::OK on success
     */
    virtual Result SetFilters(const std::vector<MessageFilter>& filters) = 0;

    /**
     * @brief Get last error message
     * @return Error string
     */
    virtual std::string GetLastError() const = 0;

    /**
     * @brief Check if driver is connected
     * @return true if connected
     */
    virtual bool IsConnected() const = 0;
};

} // namespace BusmasterCore

