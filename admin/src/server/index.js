import express from 'express';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { registerArtistRoutes } from './routes/artists.js';
import { registerCategoryRoutes } from './routes/categories.js';
import { registerStationRoutes } from './routes/stations.js';
import { registerUserRoutes } from './routes/users.js';
import { registerTrackRoutes } from './routes/tracks.js';
import { registerPlayLogRoutes } from './routes/play-log.js';
import { registerAutomixLogRoutes } from './routes/automix-log.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const adminRoot = path.resolve(__dirname, '../..');
const projectRoot = path.resolve(adminRoot, '..');
const distDir = path.join(adminRoot, 'dist');

const DEFAULT_DATABASE_CONFIG = {
  database: 'openstudio',
  host: 'localhost',
  password: 'openstudio',
  port: 5432,
  user: 'openstudio'
};

function databaseConfigPath(dataDir) {
  return path.join(dataDir, 'database.json');
}

function sanitizeDatabaseConfig(value = {}) {
  return {
    database: String(value.database || DEFAULT_DATABASE_CONFIG.database),
    host: String(value.host || DEFAULT_DATABASE_CONFIG.host),
    password: String(value.password || DEFAULT_DATABASE_CONFIG.password),
    port: Number(value.port || DEFAULT_DATABASE_CONFIG.port),
    user: String(value.user || DEFAULT_DATABASE_CONFIG.user)
  };
}

function readJson(pathname) {
  return JSON.parse(fs.readFileSync(pathname, 'utf8'));
}

function readDatabaseConfig(dataDir) {
  const storedPath = databaseConfigPath(dataDir);
  if (fs.existsSync(storedPath)) {
    return sanitizeDatabaseConfig(readJson(storedPath));
  }

  const rustConfigPath = path.join(projectRoot, 'config/database.json');
  if (fs.existsSync(rustConfigPath)) {
    return sanitizeDatabaseConfig(readJson(rustConfigPath));
  }

  return { ...DEFAULT_DATABASE_CONFIG };
}

function writeDatabaseConfig(dataDir, config) {
  fs.mkdirSync(dataDir, { recursive: true });
  fs.writeFileSync(
    databaseConfigPath(dataDir),
    JSON.stringify(sanitizeDatabaseConfig(config), null, 2)
  );
}

async function testDatabaseConnection(config) {
  try {
    const { Client } = await import('pg');
    const client = new Client(sanitizeDatabaseConfig(config));
    await client.connect();
    await client.query('SELECT 1');
    await client.end();
    return {
      connected: true,
      error: null,
      checkedAt: new Date().toISOString()
    };
  } catch (error) {
    return {
      connected: false,
      error: error.message,
      checkedAt: new Date().toISOString()
    };
  }
}

function listen(app, host, port) {
  const server = http.createServer(app);

  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, host, () => {
      server.off('error', reject);
      resolve(server);
    });
  });
}

async function attachWebUi(app) {
  const indexFile = path.join(distDir, 'index.html');

  if (fs.existsSync(indexFile)) {
    app.use(express.static(distDir));
    app.use((_req, res) => res.sendFile(indexFile));
    return;
  }

  const { createServer } = await import('vite');
  const vite = await createServer({
    root: adminRoot,
    appType: 'spa',
    server: {
      middlewareMode: true
    }
  });

  app.use(vite.middlewares);
}

export async function createOpenStudioAdminServer(config) {
  const host = config.bindAddress || '127.0.0.1';
  const webPort = Number(config.webPort || 7061);
  const controlPort = Number(config.controlPort || 7063);
  const dataDir = config.dataDir || path.join(adminRoot, '.data');
  let databaseStatus = await testDatabaseConnection(readDatabaseConfig(dataDir));

  const webApp = express();
  webApp.disable('x-powered-by');
  webApp.use(express.json());
  webApp.get('/api/hello', (_req, res) => {
    res.json({
      app: 'OpenStudio Admin',
      status: 'ok'
    });
  });
  webApp.get('/api/database', (_req, res) => {
    res.json({
      config: readDatabaseConfig(dataDir),
      status: databaseStatus
    });
  });
  webApp.get('/api/database/status', (_req, res) => {
    res.json(databaseStatus);
  });
  webApp.post('/api/database', async (req, res) => {
    const databaseConfig = sanitizeDatabaseConfig(req.body);
    writeDatabaseConfig(dataDir, databaseConfig);
    databaseStatus = await testDatabaseConnection(databaseConfig);
    res.json({
      config: databaseConfig,
      status: databaseStatus
    });
  });
  registerCategoryRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerArtistRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerStationRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerUserRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerTrackRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerPlayLogRoutes(webApp, () => readDatabaseConfig(dataDir));
  registerAutomixLogRoutes(webApp, () => readDatabaseConfig(dataDir));
  await attachWebUi(webApp);

  const controlApp = express();
  controlApp.disable('x-powered-by');
  controlApp.get('/health', (_req, res) => {
    res.json({
      app: 'OpenStudio Admin Launcher',
      status: 'running',
      web: `http://localhost:${webPort}`,
      bindAddress: host,
      webPort,
      controlPort
    });
  });

  const webServer = await listen(webApp, host, webPort);
  const controlServer = await listen(controlApp, host, controlPort);

  return {
    webServer,
    controlServer,
    webUrl: `http://localhost:${webPort}`,
    controlUrl: `http://localhost:${controlPort}`,
    close() {
      webServer.close();
      controlServer.close();
    }
  };
}
