# Getting Started with BUSMASTER Web

Welcome! This guide will help you set up and run BUSMASTER Web for the first time.

## 🚀 Quick Start (Recommended - Docker)

The fastest way to get started is using Docker:

```bash
# Clone the repository
cd busmaster-web

# Start both frontend and backend
docker-compose -f docker/docker-compose.yml up -d

# Access the application
# Frontend: http://localhost:3000
# API: http://localhost:8080
# API Docs: http://localhost:8080/api-docs
```

That's it! Skip to the [Using the Application](#using-the-application) section.

---

## 💻 Development Setup (Node.js Required)

If you want to develop or run without Docker, follow these steps:

### Prerequisites

1. **Node.js 20 LTS or higher**
   - Download from: https://nodejs.org/
   - Verify: `node --version` (should be v20.x.x or higher)

2. **Git** (for cloning)
   - Download from: https://git-scm.com/

3. **C++ Build Tools** (optional, for core service - Phase 2)
   - Windows: Visual Studio 2022 or Build Tools
   - Linux: GCC 11+
   - macOS: Xcode Command Line Tools

### Step 1: Clone the Repository

```bash
git clone https://github.com/yourusername/busmaster-web.git
cd busmaster-web
```

### Step 2: Setup Backend API Server

```bash
cd server

# Install dependencies
npm install

# Create environment file
copy env.example .env  # Windows
# OR
cp env.example .env    # Linux/Mac

# Start development server
npm run dev
```

The API server will start on http://localhost:8080

**Verify it's working:**
- Open http://localhost:8080/health in your browser
- You should see: `{"status":"ok","timestamp":"...","uptime":...}`

### Step 3: Setup Frontend (New Terminal)

```bash
cd web-ui

# Install dependencies
npm install

# Start development server
npm run dev
```

The frontend will start on http://localhost:3000

**Your browser should automatically open to the application!**

---

## 📱 Using the Application

### 1. Dashboard
- Overview of connection status
- Message statistics
- Quick start guide

### 2. Configuration
- Select hardware driver (Virtual CAN is available immediately)
- Configure channel settings
- Click "Connect" to start

### 3. Message Window
- View real-time CAN messages
- Pause/Resume message display
- Clear message buffer
- (In development mode, simulated messages will appear)

### 4. Transmit
- Send custom CAN messages
- Enter Message ID and data bytes in hexadecimal
- Click "Send Message"

---

## 🔧 Configuration

### Backend Configuration

Edit `server/.env`:

```env
# Server settings
PORT=8080
HOST=localhost

# CORS (for frontend)
CORS_ORIGIN=http://localhost:3000

# Logging
LOG_LEVEL=debug

# Enable virtual CAN simulator
VIRTUAL_CAN_ENABLED=true
```

### Frontend Configuration

Edit `web-ui/vite.config.ts` to change API proxy settings if needed.

---

## 🧪 Testing

### Backend Tests
```bash
cd server
npm test
npm run test:coverage
```

### Frontend Tests
```bash
cd web-ui
npm test
npm run test:coverage
```

---

## 🐛 Troubleshooting

### Problem: "Cannot connect to API"

**Solution:**
1. Verify backend is running on http://localhost:8080
2. Check backend logs for errors
3. Ensure firewall isn't blocking port 8080

### Problem: "Module not found" errors

**Solution:**
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Problem: Port already in use

**Solution:**
```bash
# Find and kill the process using the port
# Windows:
netstat -ano | findstr :8080
taskkill /PID <process_id> /F

# Linux/Mac:
lsof -ti:8080 | xargs kill -9
```

### Problem: WebSocket connection fails

**Solution:**
- Check browser console for CORS errors
- Verify CORS_ORIGIN in server/.env matches your frontend URL
- Try hard refresh (Ctrl+Shift+R)

---

## 📚 Next Steps

Now that you have BUSMASTER Web running:

1. **Explore the API**
   - Open http://localhost:8080/api-docs
   - Try the interactive API documentation (Swagger)

2. **Connect Real Hardware** (Coming in Phase 2)
   - PEAK USB drivers
   - Vector XL drivers
   - ETAS BOA drivers

3. **Contribute**
   - Check out [CONTRIBUTING.md](CONTRIBUTING.md)
   - Look at open issues on GitHub
   - Submit pull requests

4. **Learn More**
   - Read the [User Guide](docs/user-guide/README.md)
   - Check the [Developer Documentation](docs/development/README.md)
   - Review the [Modernization Plan](../MODERNIZATION_PLAN.md)

---

## 🆘 Getting Help

- **Documentation**: Check the `docs/` folder
- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions
- **Email**: support@example.com

---

## ✅ Verification Checklist

Before reporting issues, verify:

- [ ] Node.js version is 20.x or higher (`node --version`)
- [ ] Backend server is running (`http://localhost:8080/health`)
- [ ] Frontend is running (`http://localhost:3000`)
- [ ] No console errors in browser Developer Tools (F12)
- [ ] Backend logs show no errors (`server/logs/combined.log`)

---

## 🎉 Success!

If you can see the Dashboard and connect using Virtual CAN, you're ready to go!

**What's Working (Phase 1 - Alpha):**
- ✅ Web UI with Material-UI
- ✅ REST API framework
- ✅ WebSocket real-time messaging
- ✅ Virtual CAN simulator (simulated messages)
- ✅ Basic message display
- ✅ Basic message transmission UI

**Coming Soon (Phase 1 - Beta):**
- Message filtering
- Database file loading
- Configuration save/load
- Hardware driver integration

**Future (Phase 2+):**
- Signal interpretation
- Node simulation
- Message logging
- Signal graphing
- Real hardware support

---

Happy testing! 🚀

