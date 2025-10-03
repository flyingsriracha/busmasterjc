# BUSMASTER Web - Project Structure

## Directory Organization

```
busmaster-web/
│
├── server/                    # Backend API Server (Node.js + Express + TypeScript)
│   ├── src/
│   │   ├── api/              # REST API routes and controllers
│   │   │   └── routes/       # Individual route modules
│   │   ├── services/         # Business logic layer
│   │   ├── websocket/        # WebSocket real-time handlers
│   │   ├── middleware/       # Express middleware
│   │   ├── utils/            # Utility functions
│   │   └── index.ts          # Server entry point
│   ├── logs/                 # Application logs (generated)
│   ├── data/                 # Database and configuration storage
│   ├── package.json          # Dependencies and scripts
│   ├── tsconfig.json         # TypeScript configuration
│   └── env.example           # Environment variables template
│
├── web-ui/                   # Frontend Web UI (React + TypeScript + Material-UI)
│   ├── src/
│   │   ├── components/       # Reusable React components
│   │   │   └── layout/       # Layout components
│   │   ├── pages/            # Page-level components
│   │   ├── features/         # Redux slices and state management
│   │   │   ├── connection/   # Connection state
│   │   │   ├── messages/     # Message buffer state
│   │   │   └── config/       # Configuration state
│   │   ├── hooks/            # Custom React hooks
│   │   ├── services/         # API client services
│   │   ├── utils/            # Utility functions
│   │   ├── types/            # TypeScript type definitions
│   │   ├── App.tsx           # Main app component
│   │   ├── main.tsx          # Application entry point
│   │   ├── store.ts          # Redux store setup
│   │   └── theme.ts          # Material-UI theme
│   ├── public/               # Static assets
│   ├── package.json          # Dependencies and scripts
│   ├── tsconfig.json         # TypeScript configuration
│   ├── vite.config.ts        # Vite build configuration
│   └── index.html            # HTML entry point
│
├── core/                     # C++ Core Library (Future - Phase 1.3)
│   ├── include/              # Public C++ headers
│   │   ├── DIL/             # Driver Interface Layer
│   │   ├── MessageBus/      # Message routing
│   │   └── Database/        # Database management
│   ├── src/                  # C++ implementation
│   ├── bindings/             # N-API bindings for Node.js
│   └── CMakeLists.txt        # CMake build configuration
│
├── drivers/                  # Hardware Driver Plugins
│   ├── virtual/              # Virtual CAN simulator
│   ├── peak/                 # PEAK USB driver wrapper
│   ├── vector/               # Vector XL driver wrapper
│   └── etas/                 # ETAS BOA driver wrapper
│
├── tests/                    # Test Suites
│   ├── backend/              # Backend API unit tests
│   ├── frontend/             # Frontend component tests
│   ├── integration/          # Integration tests
│   ├── e2e/                  # End-to-end tests (Playwright)
│   ├── fixtures/             # Test data and fixtures
│   ├── mocks/                # Mock implementations
│   ├── package.json          # Test dependencies
│   ├── vitest.config.ts      # Vitest configuration
│   ├── playwright.config.ts  # Playwright configuration
│   └── setup.ts              # Test setup
│
├── docs/                     # Documentation
│   ├── api/                  # API documentation
│   ├── user-guide/           # User guide
│   ├── development/          # Developer documentation
│   └── architecture/         # Architecture diagrams
│
├── docker/                   # Docker Configuration
│   ├── Dockerfile.server     # Backend container
│   ├── Dockerfile.ui         # Frontend container
│   ├── nginx.conf            # Nginx configuration
│   └── docker-compose.yml    # Full stack orchestration
│
├── .vscode/                  # VSCode configuration
│   ├── settings.json         # Editor settings
│   └── extensions.json       # Recommended extensions
│
├── .gitignore                # Git ignore rules
├── README.md                 # Project overview
├── GETTING_STARTED.md        # Setup and installation guide
├── QUICK_START.md            # Quick start guide
├── MODERNIZATION_PLAN.md     # Full modernization roadmap
├── PHASE1_SUMMARY.md         # Current phase progress
└── START_BUSMASTER_WEB.bat   # Windows startup script
```

## File Naming Conventions

### Backend (TypeScript)
- **Routes**: `kebab-case.ts` (e.g., `connection.ts`, `message.ts`)
- **Classes**: `PascalCase.ts` (e.g., `MessageService.ts`)
- **Utilities**: `camelCase.ts` (e.g., `logger.ts`, `errorHandler.ts`)
- **Tests**: `*.test.ts` (e.g., `connection.test.ts`)

### Frontend (React + TypeScript)
- **Components**: `PascalCase.tsx` (e.g., `MainLayout.tsx`, `MessageWindow.tsx`)
- **Pages**: `PascalCase.tsx` with "Page" suffix (e.g., `DashboardPage.tsx`)
- **Hooks**: `camelCase.ts` with "use" prefix (e.g., `useConnection.ts`)
- **Redux Slices**: `camelCaseSlice.ts` (e.g., `connectionSlice.ts`)
- **Tests**: `*.test.tsx` (e.g., `Dashboard.test.tsx`)

### C++ Core
- **Headers**: `PascalCase.h` (e.g., `MessageBus.h`, `DIL_CAN.h`)
- **Implementation**: `PascalCase.cpp` (e.g., `MessageBus.cpp`)
- **Tests**: `*Test.cpp` (e.g., `MessageBusTest.cpp`)

## Module Dependencies

```
web-ui (React)
    ↓ HTTP/WebSocket
server (Node.js)
    ↓ N-API (Future)
core (C++)
    ↓ Plugin API
drivers (DLLs)
    ↓ Hardware API
CAN/LIN Hardware
```

## Build Outputs

### Development
- `server/`: No build (runs with `tsx`)
- `web-ui/`: No build (runs with Vite dev server)

### Production
- `server/dist/`: Compiled JavaScript
- `web-ui/dist/`: Bundled static files
- `core/build/`: Compiled C++ library

## Important Files

### Configuration
- `server/env.example` - Environment variables template
- `web-ui/vite.config.ts` - Vite build and dev server config
- `tests/vitest.config.ts` - Test runner configuration

### Documentation
- `README.md` - Project overview and quick links
- `GETTING_STARTED.md` - Detailed setup instructions
- `MODERNIZATION_PLAN.md` - Complete roadmap
- `PHASE1_SUMMARY.md` - Current progress

### Automation
- `START_BUSMASTER_WEB.bat` - Windows startup script
- `docker/docker-compose.yml` - Docker deployment
- `.github/workflows/` - CI/CD pipelines (Future)

## Clean Architecture Principles

1. **Separation of Concerns**: Backend, frontend, and core are independent
2. **Single Responsibility**: Each module has one clear purpose
3. **Dependency Injection**: Services are injected, not hard-coded
4. **Testability**: All layers can be tested independently
5. **Scalability**: Easy to add new features without breaking existing code

## Next Steps

When Node.js is installed, start with:
```bash
cd server
npm install
npm run dev

# New terminal
cd web-ui
npm install
npm run dev
```

Then explore the codebase starting from:
1. `server/src/index.ts` - Backend entry point
2. `web-ui/src/App.tsx` - Frontend entry point
3. `server/src/api/routes.ts` - API routes overview
4. `web-ui/src/store.ts` - State management setup

