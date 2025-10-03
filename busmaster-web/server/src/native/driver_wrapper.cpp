/*
 * BUSMASTER Web - N-API Driver Wrapper Implementation
 */

#include "driver_wrapper.h"

Napi::Object DriverWrapper::Init(Napi::Env env, Napi::Object exports) {
    Napi::Function func = DefineClass(env, "DriverWrapper", {
        InstanceMethod("initialize", &DriverWrapper::Initialize),
        InstanceMethod("shutdown", &DriverWrapper::Shutdown),
        InstanceMethod("getDriverInfo", &DriverWrapper::GetDriverInfo),
        InstanceMethod("listHardware", &DriverWrapper::ListHardware),
        InstanceMethod("selectHardware", &DriverWrapper::SelectHardware),
        InstanceMethod("deselectHardware", &DriverWrapper::DeselectHardware),
        InstanceMethod("start", &DriverWrapper::Start),
        InstanceMethod("stop", &DriverWrapper::Stop),
        InstanceMethod("sendMessage", &DriverWrapper::SendMessage),
        InstanceMethod("getStatistics", &DriverWrapper::GetStatistics),
        InstanceMethod("getErrorCounter", &DriverWrapper::GetErrorCounter),
        InstanceMethod("isConnected", &DriverWrapper::IsConnected),
        InstanceMethod("setMessageCallback", &DriverWrapper::SetMessageCallback),
    });

    Napi::FunctionReference* constructor = new Napi::FunctionReference();
    *constructor = Napi::Persistent(func);
    env.SetInstanceData(constructor);

    exports.Set("DriverWrapper", func);
    return exports;
}

DriverWrapper::DriverWrapper(const Napi::CallbackInfo& info)
    : Napi::ObjectWrap<DriverWrapper>(info) {
    
    Napi::Env env = info.Env();
    
    // For now, create Virtual CAN driver
    m_driver = std::make_unique<VirtualCANDriver>();
}

DriverWrapper::~DriverWrapper() {
    if (m_driver) {
        m_driver->Shutdown();
    }
}

Napi::Value DriverWrapper::Initialize(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    Result result = m_driver->Initialize();
    
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::Shutdown(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    Result result = m_driver->Shutdown();
    
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::GetDriverInfo(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    DriverInfo driverInfo;
    Result result = m_driver->GetDriverInfo(driverInfo);
    
    if (result != Result::OK) {
        Napi::Error::New(env, "Failed to get driver info").ThrowAsJavaScriptException();
        return env.Null();
    }
    
    Napi::Object obj = Napi::Object::New(env);
    obj.Set("id", driverInfo.id);
    obj.Set("name", driverInfo.name);
    obj.Set("version", driverInfo.version);
    obj.Set("available", driverInfo.available);
    
    return obj;
}

Napi::Value DriverWrapper::ListHardware(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    std::vector<HardwareInterface> interfaces;
    Result result = m_driver->ListHardware(interfaces);
    
    if (result != Result::OK) {
        Napi::Error::New(env, "Failed to list hardware").ThrowAsJavaScriptException();
        return env.Null();
    }
    
    Napi::Array arr = Napi::Array::New(env, interfaces.size());
    for (size_t i = 0; i < interfaces.size(); i++) {
        Napi::Object obj = Napi::Object::New(env);
        obj.Set("id", interfaces[i].id);
        obj.Set("name", interfaces[i].name);
        obj.Set("description", interfaces[i].description);
        obj.Set("available", interfaces[i].available);
        obj.Set("channelCount", interfaces[i].channelCount);
        arr.Set(i, obj);
    }
    
    return arr;
}

Napi::Value DriverWrapper::SelectHardware(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    if (info.Length() < 2) {
        Napi::TypeError::New(env, "Expected interfaceId and config").ThrowAsJavaScriptException();
        return env.Null();
    }
    
    std::string interfaceId = info[0].As<Napi::String>().Utf8Value();
    Napi::Object configObj = info[1].As<Napi::Object>();
    
    ControllerConfig config;
    config.baudrate = configObj.Get("baudrate").As<Napi::Number>().Uint32Value();
    config.mode = ControllerMode::ACTIVE;
    config.selfReception = true;
    
    Result result = m_driver->SelectHardware(interfaceId, config);
    
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::DeselectHardware(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    Result result = m_driver->DeselectHardware();
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::Start(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    Result result = m_driver->Start();
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::Stop(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    Result result = m_driver->Stop();
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::SendMessage(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    if (info.Length() < 1) {
        Napi::TypeError::New(env, "Expected message object").ThrowAsJavaScriptException();
        return env.Null();
    }
    
    CANMessage msg = JSToCANMessage(info[0].As<Napi::Object>());
    Result result = m_driver->SendMessage(msg);
    
    return Napi::Boolean::New(env, result == Result::OK);
}

Napi::Value DriverWrapper::GetStatistics(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    NetworkStatistics stats;
    Result result = m_driver->GetStatistics(stats);
    
    if (result != Result::OK) {
        return env.Null();
    }
    
    Napi::Object obj = Napi::Object::New(env);
    obj.Set("totalMessages", Napi::Number::New(env, static_cast<double>(stats.totalMessages)));
    obj.Set("messagesPerSecond", stats.messagesPerSecond);
    obj.Set("busLoad", stats.busLoad);
    obj.Set("errorFrames", stats.errorFrames);
    obj.Set("txErrors", stats.txErrors);
    obj.Set("rxErrors", stats.rxErrors);
    
    return obj;
}

Napi::Value DriverWrapper::GetErrorCounter(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    ErrorCounter counter;
    Result result = m_driver->GetErrorCounter(counter, 0);
    
    if (result != Result::OK) {
        return env.Null();
    }
    
    Napi::Object obj = Napi::Object::New(env);
    obj.Set("txErrors", counter.txErrors);
    obj.Set("rxErrors", counter.rxErrors);
    
    return obj;
}

Napi::Value DriverWrapper::IsConnected(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    return Napi::Boolean::New(env, m_driver->IsConnected());
}

void DriverWrapper::SetMessageCallback(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    if (info.Length() < 1 || !info[0].IsFunction()) {
        Napi::TypeError::New(env, "Expected callback function").ThrowAsJavaScriptException();
        return;
    }
    
    // Create thread-safe function for callbacks
    m_messageCallback = Napi::ThreadSafeFunction::New(
        env,
        info[0].As<Napi::Function>(),
        "MessageCallback",
        0,
        1
    );
    
    // Register C++ callback
    m_driver->RegisterMessageCallback(MessageCallbackImpl, this);
}

void DriverWrapper::MessageCallbackImpl(const CANMessage& msg, void* userData) {
    DriverWrapper* wrapper = static_cast<DriverWrapper*>(userData);
    
    if (wrapper->m_messageCallback) {
        wrapper->m_messageCallback.BlockingCall([msg](Napi::Env env, Napi::Function jsCallback) {
            Napi::Object msgObj = CANMessageToJS(env, msg);
            jsCallback.Call({msgObj});
        });
    }
}

Napi::Object DriverWrapper::CANMessageToJS(Napi::Env env, const CANMessage& msg) {
    Napi::Object obj = Napi::Object::New(env);
    
    obj.Set("id", msg.id);
    obj.Set("extended", msg.extended);
    obj.Set("rtr", msg.rtr);
    obj.Set("canfd", msg.canfd);
    obj.Set("length", msg.length);
    obj.Set("channel", msg.channel);
    obj.Set("timestamp", Napi::Number::New(env, static_cast<double>(msg.timestamp)));
    obj.Set("direction", static_cast<int>(msg.direction));
    
    // Convert data bytes to array
    Napi::Array dataArr = Napi::Array::New(env, msg.length);
    for (uint8_t i = 0; i < msg.length; i++) {
        dataArr.Set(i, msg.data[i]);
    }
    obj.Set("data", dataArr);
    
    return obj;
}

CANMessage DriverWrapper::JSToCANMessage(const Napi::Object& obj) {
    CANMessage msg = {};
    
    msg.id = obj.Get("id").As<Napi::Number>().Uint32Value();
    msg.extended = obj.Get("extended").As<Napi::Boolean>().Value();
    msg.rtr = obj.Has("rtr") ? obj.Get("rtr").As<Napi::Boolean>().Value() : false;
    msg.canfd = obj.Has("canfd") ? obj.Get("canfd").As<Napi::Boolean>().Value() : false;
    msg.channel = obj.Has("channel") ? obj.Get("channel").As<Napi::Number>().Uint32Value() : 0;
    msg.direction = Direction::TX;
    
    // Get data array
    Napi::Array dataArr = obj.Get("data").As<Napi::Array>();
    msg.length = static_cast<uint8_t>(dataArr.Length());
    
    for (uint8_t i = 0; i < msg.length && i < 64; i++) {
        msg.data[i] = dataArr.Get(i).As<Napi::Number>().Uint32Value();
    }
    
    return msg;
}

