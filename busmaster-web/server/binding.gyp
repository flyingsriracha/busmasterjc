{
  "targets": [
    {
      "target_name": "busmaster_native",
      "sources": [
        "src/native/busmaster_addon.cpp",
        "src/native/driver_wrapper.cpp",
        "../core/src/VirtualCANDriver.cpp"
      ],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")",
        "../core/include",
        "../core/src"
      ],
      "dependencies": [
        "<!(node -p \"require('node-addon-api').gyp\")"
      ],
      "cflags!": [ "-fno-exceptions" ],
      "cflags_cc!": [ "-fno-exceptions" ],
      "defines": [ "NAPI_DISABLE_CPP_EXCEPTIONS" ],
      "conditions": [
        ["OS=='win'", {
          "msvs_settings": {
            "VCCLCompilerTool": {
              "ExceptionHandling": 1,
              "AdditionalOptions": [ "/std:c++17" ]
            }
          },
          "defines": [ "WIN32_LEAN_AND_MEAN", "NOMINMAX" ]
        }],
        ["OS=='linux'", {
          "cflags_cc": [ "-std=c++17", "-fexceptions" ]
        }],
        ["OS=='mac'", {
          "cflags_cc": [ "-std=c++17", "-fexceptions" ],
          "xcode_settings": {
            "GCC_ENABLE_CPP_EXCEPTIONS": "YES",
            "CLANG_CXX_LANGUAGE_STANDARD": "c++17"
          }
        }]
      ]
    }
  ]
}

