import { withDatabase } from '../db/client.js';
import {
  countSchedules,
  createSchedule,
  deleteSchedule,
  listSchedules,
  updateSchedule
} from '../repositories/schedules.js';
import { listTemplates } from '../repositories/formats.js';

const LIMIT = 50;

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1, 10) || 1);
  const limit = Math.min(200, Math.max(1, parseInt(query.limit || LIMIT, 10) || LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

const DAYS = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parseHour(value) {
  const h = Number(value);
  return Number.isInteger(h) && h >= 0 && h <= 23 ? h : null;
}

function validate(data) {
  const template_id = parseId(data?.template_id);
  if (!template_id) return { ok: false, error: 'Template is required.' };

  const from_hour = parseHour(data?.from_hour);
  if (from_hour === null) return { ok: false, error: 'From hour is invalid.' };

  const to_hour = parseHour(data?.to_hour);
  if (to_hour === null) return { ok: false, error: 'To hour is invalid.' };

  const days = {};
  for (const day of DAYS) {
    days[day] = Boolean(data?.[day]);
  }

  return { ok: true, value: { template_id, from_hour, to_hour, ...days } };
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

export function registerScheduleRoutes(app, getDatabaseConfig) {
  app.get('/api/schedules/options', asyncRoute(async (_req, res) => {
    const templates = await withDatabase(getDatabaseConfig(), (db) => listTemplates(db));
    res.json({ templates });
  }));

  app.get('/api/schedules', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countSchedules(db), listSchedules(db, { limit, offset })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/schedules', asyncRoute(async (req, res) => {
    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createSchedule(db, result.value)
    );
    res.status(201).json({ row });
  }));

  app.put('/api/schedules/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid schedule id.' }); return; }

    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateSchedule(db, id, result.value)
    );
    if (!row) { res.status(404).json({ error: 'Schedule not found.' }); return; }

    res.json({ row });
  }));

  app.delete('/api/schedules/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid schedule id.' }); return; }

    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteSchedule(db, id));
    if (!deleted) { res.status(404).json({ error: 'Schedule not found.' }); return; }

    res.status(204).send();
  }));
}
