import { withDatabase } from '../db/client.js';
import {
  deleteQueueInPeriod,
  getConfiguredTimezone,
  listQueueCoverage
} from '../repositories/playlists.js';
import {
  getQueueGenerationJob,
  startQueueGeneration
} from '../services/queue-generator.js';

const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

function validateRange(data) {
  const fromDate = String(data?.from_date || '').trim();
  const toDate = String(data?.to_date || '').trim();
  const fromHour = parseHour(data?.from_hour);
  const toHour = parseHour(data?.to_hour);

  if (!DATE_RE.test(fromDate)) return { ok: false, error: 'Start date is required.' };
  if (!DATE_RE.test(toDate)) return { ok: false, error: 'End date is required.' };
  if (fromHour === null) return { ok: false, error: 'Start hour is invalid.' };
  if (toHour === null) return { ok: false, error: 'End hour is invalid.' };
  if (toDate < fromDate || (toDate === fromDate && toHour < fromHour)) {
    return { ok: false, error: 'End must be after start.' };
  }

  return { ok: true, value: { fromDate, fromHour, toDate, toHour } };
}

function parseHour(value) {
  const hour = Number(value);
  return Number.isInteger(hour) && hour >= 0 && hour <= 23 ? hour : null;
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

export function registerPlaylistRoutes(app, getDatabaseConfig) {
  app.get('/api/playlists/coverage', asyncRoute(async (req, res) => {
    const days = parseDays(req.query.days);
    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getConfiguredTimezone(db);
      const rows = await listQueueCoverage(db, timezone, days);
      return { timezone, rows };
    });
    res.json(payload);
  }));

  app.post('/api/playlists/generate', asyncRoute(async (req, res) => {
    const result = validateRange(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const job = startQueueGeneration(getDatabaseConfig(), result.value);
    res.status(202).json({ job });
  }));

  app.get('/api/playlists/generate/:id', asyncRoute(async (req, res) => {
    const job = getQueueGenerationJob(req.params.id);
    if (!job) { res.status(404).json({ error: 'Generation job not found.' }); return; }

    res.json({ job });
  }));

  app.delete('/api/playlists/queue', asyncRoute(async (req, res) => {
    const result = validateRange(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const deleted = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getConfiguredTimezone(db);
      return deleteQueueInPeriod(
        db,
        hourBoundary(result.value.fromDate, result.value.fromHour),
        hourBoundary(result.value.toDate, result.value.toHour),
        timezone
      );
    });

    res.json({ deleted });
  }));
}

function hourBoundary(date, hour) {
  return `${date} ${String(hour).padStart(2, '0')}:00:00`;
}

function parseDays(value) {
  const days = Number(value || 42);
  return Number.isInteger(days) && days >= 7 && days <= 84 ? days : 42;
}
