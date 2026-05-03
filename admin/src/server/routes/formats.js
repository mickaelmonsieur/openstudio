import { withDatabase } from '../db/client.js';
import {
  createTemplate,
  deleteTemplate,
  getTemplate,
  listTemplates,
  updateTemplate
} from '../repositories/formats.js';
import {
  createTemplateSlot,
  deleteTemplateSlot,
  getSlotOptions,
  listSlotsForTemplate,
  reorderTemplateSlots,
  updateTemplateSlot
} from '../repositories/templates.js';

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function validate(data) {
  const name = String(data?.name || '').trim();
  if (!name) return { ok: false, error: 'Name is required.' };
  if (name.length > 32) return { ok: false, error: 'Name must be 32 characters or less.' };
  return { ok: true, value: { name } };
}

function validateSlot(data) {
  const category_id = parseId(data?.category_id);
  if (!category_id) return { ok: false, error: 'Category is required.' };

  const subcategory_id = data?.subcategory_id ? parseId(data.subcategory_id) : null;
  if (data?.subcategory_id && !subcategory_id) return { ok: false, error: 'Invalid subcategory.' };

  const comment = String(data?.comment || '').trim();
  if (comment.length > 64) return { ok: false, error: 'Comment must be 64 characters or less.' };

  const track_protection = Number(data?.track_protection ?? 3600);
  if (!Number.isInteger(track_protection) || track_protection < 0) {
    return { ok: false, error: 'Track protection must be a non-negative integer.' };
  }

  const artist_protection = Number(data?.artist_protection ?? 3600);
  if (!Number.isInteger(artist_protection) || artist_protection < 0) {
    return { ok: false, error: 'Artist protection must be a non-negative integer.' };
  }

  const insert_after_id = data?.insert_after_id ? parseId(data.insert_after_id) : null;
  if (data?.insert_after_id && !insert_after_id) return { ok: false, error: 'Invalid insert position.' };

  return { ok: true, value: { category_id, subcategory_id, comment, track_protection, artist_protection, insert_after_id } };
}

function validateSlotOrder(data) {
  if (!Array.isArray(data?.ids)) return { ok: false, error: 'Slot ids are required.' };

  const ids = data.ids.map(parseId);
  if (ids.some((id) => !id)) return { ok: false, error: 'Invalid slot id in order.' };
  if (new Set(ids).size !== ids.length) return { ok: false, error: 'Duplicate slot id in order.' };

  return { ok: true, value: ids };
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

export function registerTemplateRoutes(app, getDatabaseConfig) {
  app.get('/api/templates', asyncRoute(async (_req, res) => {
    const rows = await withDatabase(getDatabaseConfig(), (db) => listTemplates(db));
    res.json({ rows });
  }));

  app.get('/api/templates/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) => getTemplate(db, id));
    if (!row) { res.status(404).json({ error: 'Template not found.' }); return; }

    res.json({ row });
  }));

  app.post('/api/templates', asyncRoute(async (req, res) => {
    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) => createTemplate(db, result.value));
    res.status(201).json({ row });
  }));

  app.put('/api/templates/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const result = validate(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) => updateTemplate(db, id, result.value));
    if (!row) { res.status(404).json({ error: 'Template not found.' }); return; }

    res.json({ row });
  }));

  app.delete('/api/templates/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteTemplate(db, id));
    if (!deleted) { res.status(404).json({ error: 'Template not found.' }); return; }

    res.status(204).send();
  }));

  // Template Slots
  app.get('/api/template-slots/options', asyncRoute(async (_req, res) => {
    const options = await withDatabase(getDatabaseConfig(), (db) => getSlotOptions(db));
    res.json(options);
  }));

  app.get('/api/templates/:id/slots', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const rows = await withDatabase(getDatabaseConfig(), (db) => listSlotsForTemplate(db, id));
    res.json({ rows });
  }));

  app.post('/api/templates/:id/slots', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const result = validateSlot(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    try {
      const row = await withDatabase(getDatabaseConfig(), (db) => createTemplateSlot(db, id, result.value));
      res.status(201).json({ row });
    } catch (error) {
      if (error.statusCode) {
        res.status(error.statusCode).json({ error: error.message });
        return;
      }
      throw error;
    }
  }));

  app.put('/api/templates/:id/slots/order', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid template id.' }); return; }

    const result = validateSlotOrder(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    try {
      const rows = await withDatabase(getDatabaseConfig(), (db) => reorderTemplateSlots(db, id, result.value));
      res.json({ rows });
    } catch (error) {
      if (error.statusCode) {
        res.status(error.statusCode).json({ error: error.message });
        return;
      }
      throw error;
    }
  }));

  app.put('/api/template-slots/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid slot id.' }); return; }

    const result = validateSlot(req.body);
    if (!result.ok) { res.status(400).json({ error: result.error }); return; }

    const row = await withDatabase(getDatabaseConfig(), (db) => updateTemplateSlot(db, id, result.value));
    if (!row) { res.status(404).json({ error: 'Slot not found.' }); return; }

    res.json({ row });
  }));

  app.delete('/api/template-slots/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid slot id.' }); return; }

    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteTemplateSlot(db, id));
    if (!deleted) { res.status(404).json({ error: 'Slot not found.' }); return; }

    res.status(204).send();
  }));
}
