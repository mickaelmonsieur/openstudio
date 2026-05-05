import { withDatabase } from '../db/client.js';

const ALLOWED_TARGETS = ['queue_played', 'play_log', 'automix_log'];
const ALLOWED_DAYS = [30, 90, 180, 365];

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

export function registerPurgeRoutes(app, getDatabaseConfig) {
  app.post('/api/purge', asyncRoute(async (req, res) => {
    const { targets, days } = req.body;

    if (!Array.isArray(targets) || targets.length === 0) {
      res.status(400).json({ error: 'No targets specified.' });
      return;
    }
    if (targets.some((t) => !ALLOWED_TARGETS.includes(t))) {
      res.status(400).json({ error: 'Invalid target.' });
      return;
    }
    if (!ALLOWED_DAYS.includes(Number(days))) {
      res.status(400).json({ error: 'Invalid days value.' });
      return;
    }

    const deleted = await withDatabase(getDatabaseConfig(), async (db) => {
      const counts = {};

      if (targets.includes('queue_played')) {
        const r = await db.query(
          `DELETE FROM queue WHERE created_at < NOW() - $1::interval RETURNING id`,
          [`${Number(days)} days`]
        );
        counts.queue_played = r.rowCount;
      }

      if (targets.includes('play_log')) {
        const r = await db.query(
          `DELETE FROM play_log WHERE played_at < NOW() - $1::interval RETURNING id`,
          [`${Number(days)} days`]
        );
        counts.play_log = r.rowCount;
      }

      if (targets.includes('automix_log')) {
        const r = await db.query(
          `DELETE FROM automix_log WHERE logged_at < NOW() - $1::interval RETURNING id`,
          [`${Number(days)} days`]
        );
        counts.automix_log = r.rowCount;
      }

      return counts;
    });

    res.json({ deleted });
  }));
}
