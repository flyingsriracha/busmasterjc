cmake_minimum_required(VERSION 3.20)
project(BusmasterCore VERSION 0.1.0 LANGUAGES CXX)

# C++ Standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Compiler flags
if(MSVC)
    add_compile_options(/W4 /WX)
else()
    add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# Source files
set(SOURCES
    src/VirtualCANDriver.cpp
)

# Header files
set(HEADERS
    include/BusmasterTypes.h
    include/IDriverInterface.h
    src/VirtualCANDriver.h
)

# Create shared library
add_library(BusmasterCore SHARED ${SOURCES} ${HEADERS})

# Set library properties
set_target_properties(BusmasterCore PROPERTIES
    VERSION ${PROJECT_VERSION}
    SOVERSION 0
    PUBLIC_HEADER "include/BusmasterTypes.h;include/IDriverInterface.h"
)

# Platform-specific settings
if(WIN32)
    target_compile_definitions(BusmasterCore PRIVATE BUSMASTER_EXPORTS)
    # Remove Windows-specific dependencies
    target_compile_definitions(BusmasterCore PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN)
endif()

# Installation
install(TARGETS BusmasterCore
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
    PUBLIC_HEADER DESTINATION include/busmaster
)

# Testing
enable_testing()

# Test executable (optional)
option(BUILD_TESTS "Build test suite" ON)
if(BUILD_TESTS)
    add_subdirectory(tests)
endif()

# Print configuration
message(STATUS "")
message(STATUS "BUSMASTER Core Configuration:")
message(STATUS "  Version: ${PROJECT_VERSION}")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Install Prefix: ${CMAKE_INSTALL_PREFIX}")
message(STATUS "")

