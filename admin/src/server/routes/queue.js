import { withDatabase } from '../db/client.js';
import {
  autocutLastInHour,
  createQueueEntryInHour,
  deleteQueueEntryFromHour,
  getQueueTimezone,
  listQueueHour,
  reorderQueueHour,
  updateQueueEntryInHour
} from '../repositories/queue.js';
import { generatePlaylistHtml, htmlToPdf } from '../services/queue-pdf.js';

const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parseHour(value) {
  const hour = Number(value);
  return Number.isInteger(hour) && hour >= 0 && hour <= 23 ? hour : null;
}

function validateHour(data) {
  const scheduled_date = String(data?.scheduled_date || data?.date || '').trim();
  if (!DATE_RE.test(scheduled_date)) return { ok: false, error: 'Date is required.' };

  const scheduled_hour = parseHour(data?.scheduled_hour ?? data?.hour);
  if (scheduled_hour === null) return { ok: false, error: 'Hour is invalid.' };

  return { ok: true, value: { scheduled_date, scheduled_hour } };
}

function validateEntry(data) {
  const hour = validateHour(data);
  if (!hour.ok) return hour;

  const track_id = parseId(data?.track_id);
  if (!track_id) return { ok: false, error: 'Track is required.' };

  const cue_in = parseNonNegativeNumber(data?.cue_in ?? 0);
  if (cue_in === null) return { ok: false, error: 'Cue In is invalid.' };

  const cue_out = parseNonNegativeNumber(data?.cue_out ?? 0);
  if (cue_out === null) return { ok: false, error: 'Cue Out is invalid.' };

  const stretch_rate = parsePositiveNumber(data?.stretch_rate ?? 1);
  if (stretch_rate === null) return { ok: false, error: 'Stretch rate is invalid.' };

  const insert_after_id = data?.insert_after_id ? parseId(data.insert_after_id) : null;
  if (data?.insert_after_id && !insert_after_id) return { ok: false, error: 'Insert position is invalid.' };

  return {
    ok: true,
    value: {
      ...hour.value,
      track_id,
      cue_in,
      cue_out,
      stretch_rate,
      insert_after_id
    }
  };
}

function validateOrder(data) {
  const hour = validateHour(data);
  if (!hour.ok) return hour;
  if (!Array.isArray(data?.ids)) return { ok: false, error: 'Queue ids are required.' };

  const ids = data.ids.map(parseId);
  if (ids.some((id) => !id)) return { ok: false, error: 'Invalid queue id in order.' };
  if (new Set(ids).size !== ids.length) return { ok: false, error: 'Duplicate queue id in order.' };

  return { ok: true, value: { ...hour.value, ids } };
}

function parseNonNegativeNumber(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
}

function parsePositiveNumber(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(error.statusCode || 500).json({ error: error.message });
    }
  };
}

export function registerQueueRoutes(app, getDatabaseConfig) {
  app.get('/queue/print', asyncRoute(async (req, res) => {
    const dateStr = String(req.query.date || '').trim();
    if (!DATE_RE.test(dateStr)) { res.status(400).send('Date invalide.'); return; }

    const mode = req.query.mode === 'day' ? 'day' : 'hour';
    let hourList;
    if (mode === 'day') {
      hourList = Array.from({ length: 24 }, (_, i) => i);
    } else {
      const h = parseHour(req.query.hour);
      if (h === null) { res.status(400).send('Heure invalide.'); return; }
      hourList = [h];
    }

    const pdf = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      const hourBlocks = [];
      for (const hour of hourList) {
        const rows = await listQueueHour(db, { date: dateStr, hour, timezone });
        if (rows.length > 0 || mode === 'hour') {
          hourBlocks.push({ date: dateStr, hour, rows });
        }
      }
      if (hourBlocks.length === 0) {
        const err = new Error('Aucune piste pour cette période.');
        err.statusCode = 404;
        throw err;
      }
      const html = generatePlaylistHtml(hourBlocks, timezone);
      return htmlToPdf(html);
    });

    const hourStr = mode === 'hour'
      ? `-${String(parseHour(req.query.hour) ?? 0).padStart(2, '0')}h`
      : '';
    res.setHeader('Content-Type', 'application/pdf');
    res.setHeader('Content-Disposition', `attachment; filename="playlist-${dateStr}${hourStr}.pdf"`);
    res.send(Buffer.from(pdf));
  }));

  app.get('/api/queue', asyncRoute(async (req, res) => {
    const result = validateHour(req.query);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      const rows = await listQueueHour(db, {
        date: result.value.scheduled_date,
        hour: result.value.scheduled_hour,
        timezone
      });
      return { rows, timezone };
    });
    res.json(payload);
  }));

  app.post('/api/queue', asyncRoute(async (req, res) => {
    const result = validateEntry(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const rows = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      return createQueueEntryInHour(db, result.value, timezone);
    });
    res.status(201).json({ rows });
  }));

  app.post('/api/queue/hour/autocut', asyncRoute(async (req, res) => {
    const result = validateHour(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      return autocutLastInHour(db, result.value.scheduled_date, result.value.scheduled_hour, timezone);
    });
    res.json({ rows: payload.rows, trimmed: payload.trimmed });
  }));

  app.put('/api/queue/hour/order', asyncRoute(async (req, res) => {
    const result = validateOrder(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const rows = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      return reorderQueueHour(
        db,
        result.value.scheduled_date,
        result.value.scheduled_hour,
        result.value.ids,
        timezone
      );
    });
    res.json({ rows });
  }));

  app.put('/api/queue/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid queue id.' }); return; }

    const result = validateEntry(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const rows = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      return updateQueueEntryInHour(db, id, result.value, timezone);
    });
    if (!rows) { res.status(404).json({ error: 'Queue entry not found.' }); return; }

    res.json({ rows });
  }));

  app.delete('/api/queue/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid queue id.' }); return; }

    const result = validateHour(req.query);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const deleted = await withDatabase(getDatabaseConfig(), async (db) => {
      const timezone = await getQueueTimezone(db);
      return deleteQueueEntryFromHour(
        db,
        id,
        result.value.scheduled_date,
        result.value.scheduled_hour,
        timezone
      );
    });
    if (!deleted) { res.status(404).json({ error: 'Queue entry not found.' }); return; }

    res.status(204).send();
  }));
}
