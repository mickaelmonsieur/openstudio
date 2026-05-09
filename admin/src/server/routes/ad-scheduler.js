import { withDatabase } from '../db/client.js';
import { getConfiguredTimezone } from '../repositories/playlists.js';
import {
  generateAdSchedule,
  listAdBreakCoverage
} from '../repositories/ad-scheduler.js';

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

function parseStationId(value) {
  const stationId = Number(value);
  return Number.isInteger(stationId) && stationId > 0 ? stationId : null;
}

function parseDays(value) {
  const days = Number(value || 7);
  return Number.isInteger(days) && days >= 7 && days <= 28 ? days : 7;
}

function parseStartDate(value) {
  const date = String(value || '').trim();
  return DATE_RE.test(date) ? date : null;
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

export function registerAdSchedulerRoutes(app, getDatabaseConfig) {
  app.get('/api/ad-scheduler/coverage', asyncRoute(async (req, res) => {
    const days = parseDays(req.query.days);
    const startDate = parseStartDate(req.query.start_date);
    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getConfiguredTimezone(db);
      const rows = await listAdBreakCoverage(db, timezone, days, startDate);
      return { timezone, start_date: startDate, rows };
    });
    res.json(payload);
  }));

  app.post('/api/ad-scheduler/generate', asyncRoute(async (req, res) => {
    const result = validateRange(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const summary = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getConfiguredTimezone(db);
      return generateAdSchedule(db, {
        ...result.value,
        timezone,
        stationId: parseStationId(req.body?.station_id)
      });
    });

    res.json({ summary });
  }));
}
