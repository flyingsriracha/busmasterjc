/*
 * BUSMASTER Web - N-API Driver Wrapper
 * 
 * Wraps C++ driver interface for Node.js
 */

#pragma once

#include <napi.h>
#include "../../core/include/IDriverInterface.h"
#include "../../core/src/VirtualCANDriver.h"
#include <memory>

using namespace BusmasterCore;

class DriverWrapper : public Napi::ObjectWrap<DriverWrapper> {
public:
    static Napi::Object Init(Napi::Env env, Napi::Object exports);
    DriverWrapper(const Napi::CallbackInfo& info);
    ~DriverWrapper();

private:
    // Driver instance
    std::unique_ptr<IDriverInterface> m_driver;
    
    // N-API wrapped methods
    Napi::Value Initialize(const Napi::CallbackInfo& info);
    Napi::Value Shutdown(const Napi::CallbackInfo& info);
    Napi::Value GetDriverInfo(const Napi::CallbackInfo& info);
    Napi::Value ListHardware(const Napi::CallbackInfo& info);
    Napi::Value SelectHardware(const Napi::CallbackInfo& info);
    Napi::Value DeselectHardware(const Napi::CallbackInfo& info);
    Napi::Value Start(const Napi::CallbackInfo& info);
    Napi::Value Stop(const Napi::CallbackInfo& info);
    Napi::Value SendMessage(const Napi::CallbackInfo& info);
    Napi::Value GetStatistics(const Napi::CallbackInfo& info);
    Napi::Value GetErrorCounter(const Napi::CallbackInfo& info);
    Napi::Value IsConnected(const Napi::CallbackInfo& info);
    
    // Callback handling
    Napi::ThreadSafeFunction m_messageCallback;
    void SetMessageCallback(const Napi::CallbackInfo& info);
    static void MessageCallbackImpl(const CANMessage& msg, void* userData);
    
    // Helper methods
    static Napi::Object CANMessageToJS(Napi::Env env, const CANMessage& msg);
    static CANMessage JSToCANMessage(const Napi::Object& obj);
};

