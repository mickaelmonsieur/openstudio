import {
  app,
  BrowserWindow,
  Menu,
  Tray,
  ipcMain,
  nativeImage,
  shell
} from 'electron';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createOpenStudioAdminServer } from '../server/index.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const adminRoot = path.resolve(__dirname, '../..');
const projectRoot = path.resolve(adminRoot, '..');

const DEFAULT_CONFIG = {
  bindAddress: '127.0.0.1',
  webPort: 7061,
  controlPort: 7063,
  autostart: false,
  startMinimized: false
};

let config = { ...DEFAULT_CONFIG };
let tray = null;
let settingsWindow = null;
let serverHandle = null;
let serverError = null;
let logFile = null;

function writeLog(message) {
  const line = `[${new Date().toISOString()}] ${message}\n`;
  console.log(line.trim());

  if (logFile) {
    try {
      fs.appendFileSync(logFile, line);
    } catch {
      // Console logging is the fallback.
    }
  }
}

function configPath() {
  return path.join(app.getPath('userData'), 'openstudio-admin.json');
}

function setupLogging() {
  logFile = path.join(app.getPath('userData'), 'launcher.log');
  writeLog(`OpenStudio Admin starting`);
  writeLog(`adminRoot=${adminRoot}`);
  writeLog(`projectRoot=${projectRoot}`);
  writeLog(`configPath=${configPath()}`);
}

function loadConfig() {
  try {
    const raw = fs.readFileSync(configPath(), 'utf8');
    config = { ...DEFAULT_CONFIG, ...JSON.parse(raw) };
  } catch {
    config = { ...DEFAULT_CONFIG };
  }
}

function saveConfig() {
  fs.mkdirSync(app.getPath('userData'), { recursive: true });
  fs.writeFileSync(configPath(), JSON.stringify(config, null, 2));
}

function sanitizeConfig(value) {
  return {
    bindAddress: String(value.bindAddress || DEFAULT_CONFIG.bindAddress),
    webPort: Number(value.webPort || DEFAULT_CONFIG.webPort),
    controlPort: Number(value.controlPort || DEFAULT_CONFIG.controlPort),
    autostart: Boolean(value.autostart),
    startMinimized: Boolean(value.startMinimized)
  };
}

function applyAutostart() {
  try {
    app.setLoginItemSettings({
      openAtLogin: config.autostart,
      args: config.startMinimized ? ['--minimized'] : []
    });
  } catch (error) {
    writeLog(`Unable to set login item: ${error.message}`);
  }
}

function networkOptions() {
  const options = [
    { label: 'Local only (127.0.0.1)', value: '127.0.0.1' },
    { label: 'All interfaces (0.0.0.0)', value: '0.0.0.0' }
  ];

  for (const [name, entries] of Object.entries(os.networkInterfaces())) {
    for (const entry of entries || []) {
      if (entry.family === 'IPv4' && !entry.internal) {
        options.push({
          label: `${name} (${entry.address})`,
          value: entry.address
        });
      }
    }
  }

  return options;
}

async function startServers() {
  try {
    serverHandle = await createOpenStudioAdminServer({
      ...config,
      dataDir: app.getPath('userData')
    });
    serverError = null;
    writeLog(`Servers running web=${config.webPort} control=${config.controlPort} bind=${config.bindAddress}`);
  } catch (error) {
    serverHandle = null;
    serverError = error.message;
    writeLog(`Server start failed: ${error.stack || error.message}`);
  }
  refreshTrayMenu();
}

async function restartServers() {
  if (serverHandle) {
    serverHandle.close();
    serverHandle = null;
  }

  await startServers();
}

function openAdmin() {
  writeLog(`Opening admin http://localhost:${config.webPort}`);
  shell.openExternal(`http://localhost:${config.webPort}`);
}

function createSettingsWindow() {
  if (settingsWindow) {
    writeLog('Showing existing settings window');
    settingsWindow.show();
    settingsWindow.focus();
    return;
  }

  writeLog('Creating settings window');
  settingsWindow = new BrowserWindow({
    width: 520,
    height: 560,
    title: 'OpenStudio Admin Launcher',
    minimizable: true,
    resizable: false,
    show: false,
    webPreferences: {
      preload: path.join(app.getAppPath(), 'src/launcher/preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  settingsWindow.webContents.on('did-fail-load', (_event, code, description) => {
    writeLog(`Settings load failed ${code}: ${description}`);
  });
  settingsWindow.webContents.on('render-process-gone', (_event, details) => {
    writeLog(`Settings renderer gone: ${JSON.stringify(details)}`);
  });

  settingsWindow.loadFile(path.join(__dirname, 'settings.html'));
  settingsWindow.once('ready-to-show', () => {
    writeLog('Settings window ready');
    settingsWindow?.show();
    settingsWindow?.setAlwaysOnTop(true);
    settingsWindow?.focus();
    setTimeout(() => {
      settingsWindow?.setAlwaysOnTop(false);
    }, 500);
  });
  settingsWindow.on('closed', () => {
    writeLog('Settings window closed');
    settingsWindow = null;
  });
}

function refreshTrayMenu() {
  if (!tray) {
    return;
  }

  tray.setContextMenu(
    Menu.buildFromTemplate([
      {
        label: 'Open OpenStudio Admin',
        click: openAdmin
      },
      {
        label: 'Settings',
        click: createSettingsWindow
      },
      {
        label: 'Restart Server',
        click: restartServers
      },
      { type: 'separator' },
      {
        label: `Web: localhost:${config.webPort}`,
        enabled: false
      },
      {
        label: `Control: localhost:${config.controlPort}`,
        enabled: false
      },
      { type: 'separator' },
      {
        label: 'Quit',
        click: () => app.quit()
      }
    ])
  );
}

function trayIcon() {
  const iconPath = path.join(projectRoot, 'icons/512x512.png');
  writeLog(`Loading tray icon ${iconPath}`);
  const image = nativeImage.createFromPath(iconPath);
  if (image.isEmpty()) {
    writeLog(`Tray icon not found or empty: ${iconPath}`);
    return nativeImage.createFromDataURL(
      `data:image/svg+xml;base64,${Buffer.from(`
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">
          <rect width="18" height="18" rx="4" fill="#6857d8"/>
          <text x="9" y="12" text-anchor="middle" font-family="Arial" font-size="9" fill="#fff">OS</text>
        </svg>
      `).toString('base64')}`
    );
  }

  return image.resize({ width: 18, height: 18 });
}

function createTray() {
  try {
    tray = new Tray(trayIcon());
    tray.setToolTip('OpenStudio Admin');
    tray.on('click', createSettingsWindow);
    tray.on('double-click', openAdmin);
    refreshTrayMenu();
    writeLog('Tray created');
  } catch (error) {
    tray = null;
    writeLog(`Tray creation failed: ${error.stack || error.message}`);
  }
}

function showSettingsWindow() {
  createSettingsWindow();
}

function createApplicationMenu() {
  Menu.setApplicationMenu(
    Menu.buildFromTemplate([
      {
        label: 'OpenStudio Admin',
        submenu: [
          {
            label: 'Show Launcher',
            click: createSettingsWindow
          },
          {
            label: 'Open Admin',
            click: openAdmin
          },
          { type: 'separator' },
          {
            label: 'Quit',
            click: () => app.quit()
          }
        ]
      }
    ])
  );
}

ipcMain.handle('launcher:get-state', () => ({
  config,
  networkOptions: networkOptions(),
  webUrl: `http://localhost:${config.webPort}`,
  controlUrl: `http://localhost:${config.controlPort}`,
  running: Boolean(serverHandle),
  error: serverError
}));

ipcMain.handle('launcher:save-config', async (_event, value) => {
  config = sanitizeConfig(value);
  saveConfig();
  applyAutostart();
  await restartServers();
  return {
    config,
    networkOptions: networkOptions(),
    webUrl: `http://localhost:${config.webPort}`,
    controlUrl: `http://localhost:${config.controlPort}`,
    running: Boolean(serverHandle),
    error: serverError
  };
});

ipcMain.handle('launcher:open-admin', openAdmin);
ipcMain.handle('launcher:minimize', () => settingsWindow?.minimize());
ipcMain.handle('launcher:hide', () => {
  settingsWindow?.hide();
  tray?.displayBalloon?.({
    title: 'OpenStudio Admin',
    content: 'The launcher is still running in the menu bar/tray.'
  });
});

app.whenReady().then(async () => {
  setupLogging();
  loadConfig();
  writeLog(`Loaded config ${JSON.stringify(config)}`);
  applyAutostart();
  createApplicationMenu();
  createTray();
  app.on('activate', showSettingsWindow);
  await startServers();

  if (!config.startMinimized && !process.argv.includes('--minimized')) {
    createSettingsWindow();
    openAdmin();
  }
}).catch((error) => {
  writeLog(`Fatal startup error: ${error.stack || error.message}`);
});

app.on('window-all-closed', () => {
  // Keep the launcher alive in the tray.
});

app.on('before-quit', () => {
  if (serverHandle) {
    serverHandle.close();
  }
});

process.on('uncaughtException', (error) => {
  writeLog(`Uncaught exception: ${error.stack || error.message}`);
});

process.on('unhandledRejection', (error) => {
  writeLog(`Unhandled rejection: ${error?.stack || error}`);
});
