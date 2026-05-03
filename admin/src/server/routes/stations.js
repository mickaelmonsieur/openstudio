import fs from 'node:fs/promises';
import { withDatabase } from '../db/client.js';
import {
  createStation,
  deleteStation,
  getStation,
  listStations,
  updateStation
} from '../repositories/stations.js';
import { suggestStationPath } from '../lib/platform.js';

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function validateStation(data) {
  const name = String(data?.name || '').trim();
  if (!name) {
    return { ok: false, error: 'Name is required.' };
  }
  if (name.length > 64) {
    return { ok: false, error: 'Name must be 64 characters or less.' };
  }

  const library_path = String(data?.library_path || '').trim();
  if (!library_path) {
    return { ok: false, error: 'Library path is required.' };
  }

  return { ok: true, value: { name, library_path } };
}

async function ensureDirectory(dirPath) {
  await fs.mkdir(dirPath, { recursive: true });
}

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

export function registerStationRoutes(app, getDatabaseConfig) {
  // Must be before /:id to avoid "suggest-path" being parsed as an id
  app.get('/api/stations/suggest-path', asyncRoute(async (req, res) => {
    const name = String(req.query.name || '').trim();
    res.json({ path: suggestStationPath(name) });
  }));

  app.get('/api/stations', asyncRoute(async (_req, res) => {
    const rows = await withDatabase(getDatabaseConfig(), listStations);
    res.json({ rows });
  }));

  app.get('/api/stations/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid station id.' });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) => getStation(db, id));
    if (!row) {
      res.status(404).json({ error: 'Station not found.' });
      return;
    }

    res.json({ row });
  }));

  app.post('/api/stations', asyncRoute(async (req, res) => {
    const station = validateStation(req.body);
    if (!station.ok) {
      res.status(400).json({ error: station.error });
      return;
    }

    await ensureDirectory(station.value.library_path);
    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createStation(db, station.value)
    );
    res.status(201).json({ row });
  }));

  app.put('/api/stations/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid station id.' });
      return;
    }

    const station = validateStation(req.body);
    if (!station.ok) {
      res.status(400).json({ error: station.error });
      return;
    }

    await ensureDirectory(station.value.library_path);
    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateStation(db, id, station.value)
    );
    if (!row) {
      res.status(404).json({ error: 'Station not found.' });
      return;
    }

    res.json({ row });
  }));

  app.delete('/api/stations/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid station id.' });
      return;
    }

    const deleted = await withDatabase(getDatabaseConfig(), (db) =>
      deleteStation(db, id)
    );
    if (!deleted) {
      res.status(404).json({ error: 'Station not found.' });
      return;
    }

    res.status(204).send();
  }));
}
