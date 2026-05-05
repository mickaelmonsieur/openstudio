import { withDatabase } from '../db/client.js';

const ALLOWED_TARGETS = ['tracks', 'artists'];

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

export function registerResetLastPlayedRoutes(app, getDatabaseConfig) {
  app.post('/api/reset-last-played', asyncRoute(async (req, res) => {
    const { targets } = req.body;

    if (!Array.isArray(targets) || targets.length === 0) {
      res.status(400).json({ error: 'No targets specified.' });
      return;
    }
    if (targets.some((t) => !ALLOWED_TARGETS.includes(t))) {
      res.status(400).json({ error: 'Invalid target.' });
      return;
    }

    const updated = await withDatabase(getDatabaseConfig(), async (db) => {
      const counts = {};

      if (targets.includes('tracks')) {
        const r = await db.query('UPDATE tracks SET last_played_at = NULL WHERE last_played_at IS NOT NULL');
        counts.tracks = r.rowCount;
      }

      if (targets.includes('artists')) {
        const r = await db.query('UPDATE artists SET last_broadcast_at = NULL WHERE last_broadcast_at IS NOT NULL');
        counts.artists = r.rowCount;
      }

      return counts;
    });

    res.json({ updated });
  }));
}
