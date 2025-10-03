# BUSMASTER Web - Build Instructions

Complete step-by-step instructions for building the entire project.

## System Requirements

### All Platforms
- **Node.js**: 20.x or later
- **npm**: 10.x or later
- **CMake**: 3.20 or later
- **Python**: 3.8+ (for node-gyp)

### Windows
- **Visual Studio 2019 or later** with:
  - Desktop development with C++
  - Windows SDK
- **Git for Windows** (recommended)

### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake g++ python3 nodejs npm git
```

### macOS
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake node python3 git

# Install Xcode Command Line Tools
xcode-select --install
```

## Project Structure

```
busmaster-web/
├── core/           # C++ core library
├── server/         # Node.js backend
├── web-ui/         # React frontend
├── tests/          # Test suites
├── docker/         # Docker deployment
└── docs/           # Documentation
```

## Build Process

### Step 1: Clone Repository

```bash
git clone https://github.com/BUSMASTER/busmaster-web.git
cd busmaster-web
```

### Step 2: Build C++ Core Library

```bash
cd core

# Create build directory
mkdir build
cd build

# Configure (Windows with Visual Studio)
cmake .. -G "Visual Studio 16 2019" -A x64

# Configure (Windows with MinGW)
cmake .. -G "MinGW Makefiles"

# Configure (Linux/macOS)
cmake ..

# Build
cmake --build . --config Release

# Optionally install
cmake --install .

cd ../..
```

### Step 3: Build Backend (with Native Addon)

```bash
cd server

# Install dependencies
npm install

# Build native addon (this uses node-gyp)
npm run build:native

# Build TypeScript
npm run build

cd ..
```

### Step 4: Build Frontend

```bash
cd web-ui

# Install dependencies
npm install

# Build for production (optional)
npm run build

cd ..
```

## Running in Development Mode

### Terminal 1: Backend Server

```bash
cd server
npm run dev
```

Server will start on: http://localhost:8080

### Terminal 2: Frontend Dev Server

```bash
cd web-ui
npm run dev
```

Frontend will start on: http://localhost:3000

### Windows: Quick Start Script

```cmd
REM Double-click this file or run from command prompt
START_BUSMASTER_WEB.bat
```

## Troubleshooting

### Windows: node-gyp Build Errors

**Error**: `Can't find Python executable`
```cmd
npm config set python "C:\Python311\python.exe"
```

**Error**: `Can't find Visual Studio`
```cmd
npm config set msvs_version 2019
```

### Linux: Missing Build Tools

```bash
sudo apt-get install -y build-essential cmake g++ python3-dev
```

### macOS: Xcode Issues

```bash
# Reinstall Command Line Tools
sudo rm -rf /Library/Developer/CommandLineTools
xcode-select --install
```

### Node-gyp Rebuild Failed

```bash
cd server

# Clean and rebuild
rm -rf build node_modules
npm install
npm run build:native
```

### CMake Configuration Failed

```bash
cd core/build

# Clean and reconfigure
rm -rf *
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build .
```

## Verification

### Check C++ Library

```bash
# Windows
dir core\build\Release\BusmasterCore.dll

# Linux
ls core/build/libBusmasterCore.so

# macOS
ls core/build/libBusmasterCore.dylib
```

### Check Native Addon

```bash
# Windows
dir server\build\Release\busmaster_native.node

# Linux/macOS
ls server/build/Release/busmaster_native.node
```

### Test Native Addon

```bash
cd server
node -e "const addon = require('./build/Release/busmaster_native.node'); console.log('Addon loaded successfully:', addon);"
```

## Production Build

### Build All Components

```bash
# 1. Build C++ core
cd core/build
cmake --build . --config Release

# 2. Build backend
cd ../../server
npm run build

# 3. Build frontend
cd ../web-ui
npm run build
```

### Deploy

```bash
# Use Docker Compose
cd docker
docker-compose up -d

# Or manually
cd server
npm start
```

## Docker Build (Alternative)

```bash
# Build and run with Docker
cd docker
docker-compose up --build
```

Access:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080

## Testing

```bash
# Backend tests
cd server
npm test

# Frontend tests
cd web-ui
npm test

# All tests
cd tests
npm install
npm test

# End-to-end tests
npm run test:e2e
```

## Build Optimization

### Faster Incremental Builds

```bash
# C++ core - use Ninja
cd core/build
cmake .. -G Ninja
ninja

# Backend - use ts-node-dev
cd server
npm install -D ts-node-dev
npx ts-node-dev src/index.ts
```

### Parallel Builds

```bash
# CMake with multiple cores
cmake --build . --config Release -- -j8

# npm with multiple cores
npm install --jobs=8
```

## Clean Build

```bash
# Clean everything
rm -rf core/build
rm -rf server/build
rm -rf server/node_modules
rm -rf web-ui/node_modules
rm -rf tests/node_modules

# Rebuild from scratch
cd core && mkdir build && cd build && cmake .. && cmake --build .
cd ../../server && npm install && npm run build:native
cd ../web-ui && npm install
```

## Next Steps

After successful build:

1. **Test the Application**: Open http://localhost:3000
2. **Read Documentation**: See `GETTING_STARTED.md`
3. **Run Tests**: `cd tests && npm test`
4. **Deploy**: See `docker/README.md`

## Support

- **Issues**: https://github.com/BUSMASTER/busmaster-web/issues
- **Wiki**: https://github.com/BUSMASTER/busmaster-web/wiki
- **Discussions**: https://github.com/BUSMASTER/busmaster-web/discussions

