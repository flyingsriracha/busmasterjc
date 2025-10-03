# Installing Node.js for BUSMASTER Web

## Why Node.js is Needed

BUSMASTER Web's backend API server and frontend build tools require Node.js.

## Installation Steps for Windows

### Option 1: Download Official Installer (Recommended)

1. **Go to Node.js website**: https://nodejs.org/
2. **Download**: Click "LTS" (Long Term Support) version for Windows
   - Should be Node.js 20.x or higher
3. **Run Installer**: Double-click the downloaded `.msi` file
4. **Follow Wizard**: Accept defaults, click Next → Next → Install
5. **Verify Installation**:
   ```powershell
   node --version
   npm --version
   ```
   You should see version numbers like `v20.x.x` and `10.x.x`

### Option 2: Using Winget (Windows Package Manager)

If you have Windows 11 or Windows 10 with winget:

```powershell
winget install OpenJS.NodeJS.LTS
```

### Option 3: Using Chocolatey

If you have Chocolatey installed:

```powershell
choco install nodejs-lts
```

## After Installation

**Close and reopen** your terminal/PowerShell, then verify:

```powershell
node --version   # Should show: v20.x.x or higher
npm --version    # Should show: 10.x.x or higher
```

## Then Run BUSMASTER Web

Once Node.js is installed:

```powershell
# Navigate to the project
cd C:\Users\CHJ1ANA\Documents\GitHub\busmasterjc\busmaster-web

# Install backend dependencies
cd server
npm install

# Start backend (keep this running)
npm run dev

# In a NEW terminal window:
cd C:\Users\CHJ1ANA\Documents\GitHub\busmasterjc\busmaster-web\web-ui
npm install
npm run dev
```

## Alternative: Use Docker (No Node.js Required!)

If you have Docker Desktop installed:

```powershell
cd C:\Users\CHJ1ANA\Documents\GitHub\busmasterjc\busmaster-web
docker-compose -f docker\docker-compose.yml up
```

Then open: http://localhost:3000

## Troubleshooting

**"Node.js is installed but command not found"**
- Close and reopen PowerShell
- Or add to PATH: `C:\Program Files\nodejs`

**"npm install fails with permission errors"**
- Run PowerShell as Administrator
- Or use: `npm install --force`

**"Port 8080 or 3000 already in use"**
- Find and close the process using that port
- Or change ports in `.env` files

## Need Help?

Check the full guide: [GETTING_STARTED.md](GETTING_STARTED.md)

