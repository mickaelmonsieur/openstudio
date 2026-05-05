import { withDatabase } from '../db/client.js';
import {
  countEvents,
  createEvent,
  deleteEvent,
  listEvents,
  updateEvent
} from '../repositories/events.js';
import { listTemplates } from '../repositories/formats.js';

const LIMIT = 50;

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1, 10) || 1);
  const limit = Math.min(200, Math.max(1, parseInt(query.limit || LIMIT, 10) || LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parseBoundedInt(value, min, max) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= min && parsed <= max ? parsed : null;
}

function parseNonNegativeNumber(value) {
  const parsed = Number(value ?? 0);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
}

function parseDate(value) {
  if (!value || typeof value !== 'string') return null;
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) return null;
  const [, y, m, d] = match;
  const date = new Date(`${y}-${m}-${d}T00:00:00Z`);
  return Number.isNaN(date.getTime()) ? null : { full: `${y}-${m}-${d}`, month: m, day: d };
}

function validate(data) {
  const event_type = parseBoundedInt(data?.event_type, 1, 5);
  if (event_type === null) return { ok: false, error: 'Invalid event type.' };

  const minute = parseBoundedInt(data?.minute, 0, 59);
  if (minute === null) return { ok: false, error: 'Minute must be between 0 and 59.' };

  const second = parseBoundedInt(data?.second, 0, 59);
  if (second === null) return { ok: false, error: 'Second must be between 0 and 59.' };

  const template_id = data?.template_id ? parseId(data.template_id) : null;
  if (data?.template_id && !template_id) return { ok: false, error: 'Invalid template.' };

  const priority = parseBoundedInt(data?.priority ?? 0, -32768, 32767);
  if (priority === null) return { ok: false, error: 'Priority must be an integer.' };

  const duration = parseNonNegativeNumber(data?.duration ?? 0);
  if (duration === null) return { ok: false, error: 'Duration must be zero or greater.' };

  // hour: used by types 1, 2, 4 — ignored (forced 0) for types 3, 5
  const hourNeeded = [1, 2, 4].includes(event_type);
  const hour = hourNeeded ? parseBoundedInt(data?.hour, 0, 23) : 0;
  if (hourNeeded && hour === null) return { ok: false, error: 'Hour must be between 0 and 23.' };

  // days_mask: required for types 2, 3
  const days_mask = Number(data?.days_mask ?? 0);
  if ([2, 3].includes(event_type) && !(Number.isInteger(days_mask) && days_mask > 0 && days_mask <= 127)) {
    return { ok: false, error: 'Select at least one day.' };
  }

  // hours_mask: required for types 3, 5
  const hours_mask = Number(data?.hours_mask ?? 0);
  if ([3, 5].includes(event_type) && !(Number.isInteger(hours_mask) && hours_mask > 0)) {
    return { ok: false, error: 'Select at least one hour.' };
  }

  // event_date: required for types 1, 4, 5
  let event_date = null;
  if ([1, 4, 5].includes(event_type)) {
    const parsed = parseDate(data?.event_date);
    if (!parsed) return { ok: false, error: 'A valid date is required.' };
    // Types 4 & 5: year is ignored — normalize to 2000
    event_date = [4, 5].includes(event_type)
      ? `2000-${parsed.month}-${parsed.day}`
      : parsed.full;
  }

  return {
    ok: true,
    value: {
      event_type,
      days_mask:  [2, 3].includes(event_type) ? days_mask  : 0,
      hours_mask: [3, 5].includes(event_type) ? hours_mask : 0,
      event_date,
      hour,
      minute,
      second,
      template_id,
      priority,
      duration
    }
  };
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

export function registerEventRoutes(app, getDatabaseConfig) {
  app.get('/api/events/options', asyncRoute(async (_req, res) => {
    const templates = await withDatabase(getDatabaseConfig(), (db) => listTemplates(db));
    res.json({ templates });
  }));

  app.get('/api/events', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countEvents(db), listEvents(db, { limit, offset })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/events', asyncRoute(async (req, res) => {
    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createEvent(db, result.value));
    res.status(201).json({ row });
  }));

  app.put('/api/events/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid event id.' }); return; }
    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => updateEvent(db, id, result.value));
    if (!row) { res.status(404).json({ error: 'Event not found.' }); return; }
    res.json({ row });
  }));

  app.delete('/api/events/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid event id.' }); return; }
    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteEvent(db, id));
    if (!deleted) { res.status(404).json({ error: 'Event not found.' }); return; }
    res.status(204).send();
  }));
}
