import { withDatabase } from '../db/client.js';
import {
  countTracks,
  deleteTrack,
  hasScheduledQueue,
  listSubcategoriesWithCategory,
  listTracks,
  updateTrack
} from '../repositories/tracks.js';

const DEFAULT_LIMIT = 100;
const MAX_LIMIT = 500;

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1,             10) || 1);
  const limit = Math.min(MAX_LIMIT, Math.max(1, parseInt(query.limit || DEFAULT_LIMIT, 10) || DEFAULT_LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

function validateTrack(data) {
  const title = String(data?.title || '').trim();
  if (!title) {
    return { ok: false, error: 'Title is required.' };
  }
  if (title.length > 64) {
    return { ok: false, error: 'Title must be 64 characters or less.' };
  }

  const album = String(data?.album || '').trim();
  if (album.length > 64) {
    return { ok: false, error: 'Album must be 64 characters or less.' };
  }

  const artist_id      = data?.artist_id      ? parseInt(data.artist_id,      10) : null;
  const subcategory_id = data?.subcategory_id  ? parseInt(data.subcategory_id, 10) : null;
  const year           = data?.year            ? parseInt(data.year,           10) : null;
  const active         = Boolean(data?.active);

  return { ok: true, value: { title, album, artist_id, subcategory_id, year, active } };
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

export function registerTrackRoutes(app, getDatabaseConfig) {
  app.get('/api/tracks', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);

    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countTracks(db), listTracks(db, { limit, offset })])
    );

    res.json({ rows, total, page, limit });
  }));

  // Must be declared before /api/tracks/:id to avoid "options" being parsed as an id
  app.get('/api/tracks/options', asyncRoute(async (_req, res) => {
    const subcategories = await withDatabase(getDatabaseConfig(), listSubcategoriesWithCategory);
    res.json({ subcategories });
  }));

  app.put('/api/tracks/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid track id.' });
      return;
    }

    const track = validateTrack(req.body);
    if (!track.ok) {
      res.status(400).json({ error: track.error });
      return;
    }

    const updated = await withDatabase(getDatabaseConfig(), (db) =>
      updateTrack(db, id, track.value)
    );
    if (!updated) {
      res.status(404).json({ error: 'Track not found.' });
      return;
    }

    res.json({ ok: true });
  }));

  app.delete('/api/tracks/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid track id.' });
      return;
    }

    const scheduled = await withDatabase(getDatabaseConfig(), (db) =>
      hasScheduledQueue(db, id)
    );
    if (scheduled) {
      res.status(409).json({
        error: 'This track is scheduled in a future queue entry and cannot be deleted.'
      });
      return;
    }

    const deleted = await withDatabase(getDatabaseConfig(), (db) =>
      deleteTrack(db, id)
    );
    if (!deleted) {
      res.status(404).json({ error: 'Track not found.' });
      return;
    }

    res.status(204).send();
  }));
}
