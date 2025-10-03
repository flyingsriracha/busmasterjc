/*
 * BUSMASTER Web - N-API Native Addon
 * 
 * Binds C++ core library to Node.js
 */

#include <napi.h>
#include "driver_wrapper.h"

// Initialize the addon
Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
    DriverWrapper::Init(env, exports);
    return exports;
}

NODE_API_MODULE(busmaster_native, InitAll)

