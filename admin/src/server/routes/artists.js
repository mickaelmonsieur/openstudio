import { withDatabase } from '../db/client.js';
import {
  createArtist,
  deleteArtist,
  getArtist,
  listArtists,
  updateArtist
} from '../repositories/artists.js';

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function validateArtist(data) {
  const name = String(data?.name || '').trim();
  if (!name) {
    return { ok: false, error: 'Name is required.' };
  }

  if (name.length > 64) {
    return { ok: false, error: 'Name must be 64 characters or less.' };
  }

  return { ok: true, value: { name } };
}

function parseSearch(query) {
  return String(query.q || '').trim().slice(0, 120);
}

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({
        error: error.message
      });
    }
  };
}

export function registerArtistRoutes(app, getDatabaseConfig) {
  app.get('/api/artists', asyncRoute(async (req, res) => {
    const search = parseSearch(req.query);
    const rows = await withDatabase(getDatabaseConfig(), (db) => listArtists(db, search));
    res.json({ rows, q: search });
  }));

  app.get('/api/artists/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid artist id.' });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) => getArtist(db, id));
    if (!row) {
      res.status(404).json({ error: 'Artist not found.' });
      return;
    }

    res.json({ row });
  }));

  app.post('/api/artists', asyncRoute(async (req, res) => {
    const artist = validateArtist(req.body);
    if (!artist.ok) {
      res.status(400).json({ error: artist.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createArtist(db, artist.value)
    );
    res.status(201).json({ row });
  }));

  app.put('/api/artists/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid artist id.' });
      return;
    }

    const artist = validateArtist(req.body);
    if (!artist.ok) {
      res.status(400).json({ error: artist.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateArtist(db, id, artist.value)
    );
    if (!row) {
      res.status(404).json({ error: 'Artist not found.' });
      return;
    }

    res.json({ row });
  }));

  app.delete('/api/artists/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid artist id.' });
      return;
    }

    const deleted = await withDatabase(getDatabaseConfig(), (db) =>
      deleteArtist(db, id)
    );
    if (!deleted) {
      res.status(404).json({ error: 'Artist not found.' });
      return;
    }

    res.status(204).send();
  }));
}
