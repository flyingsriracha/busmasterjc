@echo off
echo ========================================
echo   BUSMASTER Web - Startup Script
echo ========================================
echo.

REM Add Node.js to PATH
set PATH=%PATH%;C:\Program Files\nodejs

REM Check if Node.js is available
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Node.js is not installed or not in PATH
    echo.
    echo Please install Node.js from: https://nodejs.org/
    echo.
    echo After installation:
    echo 1. Close this window
    echo 2. Open a NEW PowerShell/Command Prompt
    echo 3. Run this script again
    echo.
    pause
    exit /b 1
)

echo [OK] Node.js is installed: 
node --version
npm --version
echo.

echo ========================================
echo   Installing Dependencies...
echo ========================================

echo [1/2] Installing backend dependencies...
cd /d "%~dp0server"
call npm install
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Backend installation failed
    pause
    exit /b 1
)
echo [OK] Backend dependencies installed
echo.

echo [2/2] Installing frontend dependencies...
cd /d "%~dp0web-ui"
call npm install
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Frontend installation failed
    pause
    exit /b 1
)
echo [OK] Frontend dependencies installed
echo.

echo ========================================
echo   Starting BUSMASTER Web...
echo ========================================
echo.
echo Starting servers in new windows...
echo.
echo [Backend] API Server will start on: http://localhost:8080
echo [Frontend] Web UI will start on: http://localhost:3000
echo.
echo IMPORTANT: Two new windows will open
echo - Keep both windows open while using BUSMASTER Web
echo - Your browser should open automatically
echo.

REM Start backend in new window
start "BUSMASTER Web - Backend API" cmd /k "cd /d %~dp0server && npm run dev"

REM Wait a moment for backend to start
timeout /t 3 /nobreak >nul

REM Start frontend in new window
start "BUSMASTER Web - Frontend" cmd /k "cd /d %~dp0web-ui && npm run dev"

echo.
echo ========================================
echo   BUSMASTER Web is Starting!
echo ========================================
echo.
echo Two new windows have opened:
echo   1. Backend API Server (port 8080)
echo   2. Frontend Web UI (port 3000)
echo.
echo Your browser should open automatically to:
echo   http://localhost:3000
echo.
echo If not, manually open: http://localhost:3000
echo.
echo To stop: Close both server windows
echo.
echo ========================================
pause

