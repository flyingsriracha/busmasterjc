# BUSMASTER Web - Modern CAN/LIN Bus Analysis Tool

**A modern, web-based rewrite of BUSMASTER for cross-platform CAN/LIN bus monitoring and analysis.**

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Status: Alpha](https://img.shields.io/badge/Status-Alpha-orange.svg)]()

## 🚀 Quick Start

### Prerequisites
- Node.js 20 LTS or higher
- CMake 3.20+
- C++ compiler (MSVC/GCC/Clang)
- Docker (optional, for containerized deployment)

### Development Setup

```bash
# 1. Setup backend API server
cd server
npm install
npm run dev

# 2. Setup frontend (in new terminal)
cd web-ui
npm install
npm start

# 3. Access the application
# Open browser to: http://localhost:3000
```

## 📁 Project Structure

```
busmaster-web/
├── core/                 # C++ core library (hardware interface, message processing)
│   ├── include/         # Public C++ headers
│   ├── src/             # C++ implementation
│   └── CMakeLists.txt   # CMake build configuration
│
├── server/              # Node.js API server
│   ├── src/
│   │   ├── api/         # REST API routes
│   │   ├── services/    # Business logic
│   │   ├── websocket/   # Real-time WebSocket handlers
│   │   └── native/      # N-API bindings to C++ core
│   ├── package.json
│   └── tsconfig.json
│
├── web-ui/              # React frontend
│   ├── src/
│   │   ├── components/  # React components
│   │   ├── pages/       # Page components
│   │   ├── hooks/       # Custom React hooks
│   │   ├── services/    # API client services
│   │   └── utils/       # Utility functions
│   ├── package.json
│   └── tsconfig.json
│
├── drivers/             # Hardware driver plugins
│   ├── peak/            # PEAK USB driver
│   ├── vector/          # Vector XL driver
│   └── virtual/         # Virtual CAN simulator
│
├── tests/               # Test suites
│   ├── unit/            # Unit tests
│   ├── integration/     # Integration tests
│   └── e2e/             # End-to-end tests
│
├── docs/                # Documentation
│   ├── api/             # API documentation
│   ├── user-guide/      # User guide
│   └── development/     # Developer documentation
│
└── docker/              # Docker configurations
    ├── Dockerfile.server
    ├── Dockerfile.ui
    └── docker-compose.yml
```

## 🎯 Features

### ✅ Currently Implemented (Phase 1 - Alpha)
- [x] Project structure and build system
- [x] Basic REST API framework
- [x] Frontend scaffold with React + TypeScript
- [ ] Hardware driver abstraction (in progress)
- [ ] Message send/receive (in progress)

### 🚧 In Development (Phase 1)
- Message display in real-time
- Basic CAN message transmission
- Virtual CAN simulator
- Configuration management

### 📋 Planned (Phase 2+)
- Signal interpretation with DBC files
- Database editor (DBF/DBC)
- Node simulation with JavaScript
- Message logging and replay
- Signal graphing and monitoring
- Cross-platform hardware support (Linux SocketCAN, etc.)

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│     Web UI (React + TypeScript)     │
│  • Message Display                  │
│  • Signal Visualization             │
│  • Configuration Management         │
└────────────────┬────────────────────┘
                 │ REST API + WebSocket
┌────────────────┴────────────────────┐
│   API Server (Node.js + Express)    │
│  • REST Endpoints                   │
│  • WebSocket Streaming              │
│  • N-API Bindings to C++            │
└────────────────┬────────────────────┘
                 │ Native Interface
┌────────────────┴────────────────────┐
│    Core Service (C++ Library)       │
│  • Hardware Interface Layer         │
│  • Message Processing               │
│  • Database Management              │
└────────────────┬────────────────────┘
                 │ Driver Plugin API
┌────────────────┴────────────────────┐
│       Hardware Drivers (DLL)        │
│  • PEAK USB • Vector XL • Virtual   │
└─────────────────────────────────────┘
```

## 🔧 Technology Stack

### Backend
- **Core**: C++17 with CMake
- **API Server**: Node.js 20 + Express.js + TypeScript
- **WebSocket**: Socket.IO
- **Database**: SQLite (config), Redis (optional cache)
- **API Docs**: OpenAPI 3.0 (Swagger)

### Frontend
- **Framework**: React 18 + TypeScript
- **UI Library**: Material-UI (MUI)
- **State Management**: Redux Toolkit + RTK Query
- **Real-time**: Socket.IO Client
- **Charts**: Recharts
- **Build**: Vite

### DevOps
- **Containerization**: Docker + Docker Compose
- **CI/CD**: GitHub Actions
- **Testing**: Jest + Vitest + Playwright
- **Linting**: ESLint + Prettier

## 🚀 Development Roadmap

See [MODERNIZATION_PLAN.md](../MODERNIZATION_PLAN.md) for the complete multi-stage implementation plan.

### Phase 1: Foundation (Months 1-3) - **CURRENT**
- ✅ Project structure setup
- 🔄 Core service extraction
- 🔄 REST API development
- 🔄 Minimal web UI

### Phase 2: Feature Parity (Months 4-8)
- Advanced message handling
- Database & configuration management
- Node simulation with JavaScript
- Hardware abstraction layer

### Phase 3: Production Ready (Months 9-12)
- UI/UX polish
- Comprehensive testing
- Documentation
- Community engagement

## 📖 Documentation

- [API Reference](docs/api/README.md)
- [User Guide](docs/user-guide/README.md)
- [Developer Guide](docs/development/README.md)
- [Hardware Driver Development](docs/development/drivers.md)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Setting up Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/busmaster-web.git
cd busmaster-web

# Install dependencies
npm run install:all

# Start development servers
npm run dev
```

## 📝 License

This project is licensed under the GNU Lesser General Public License v3.0 (LGPL-3.0) - see the [LICENSE](../COPYING.LESSER.txt) file for details.

This is a modernization of the original BUSMASTER project by RBEI-ETAS.

## 🙏 Acknowledgments

- Original BUSMASTER team at RBEI-ETAS/Bosch
- All contributors to the legacy BUSMASTER project
- Community members who provided feedback through 974 GitHub issues

## 📧 Contact & Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/busmaster-web/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/busmaster-web/discussions)
- **Email**: support@example.com

---

**Status**: 🚧 Active Development - Alpha Release  
**Current Phase**: Phase 1 - Foundation & Proof of Concept  
**Target Release**: Q2 2026

