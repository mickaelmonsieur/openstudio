import { withDatabase } from '../db/client.js';

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

export function registerOptimizeRoutes(app, getDatabaseConfig) {
  app.post('/api/optimize', asyncRoute(async (req, res) => {
    const config = getDatabaseConfig();

    await withDatabase(config, async (db) => {
      await db.query('VACUUM FULL ANALYZE');
      await db.query(`REINDEX DATABASE "${config.database}"`);
    });

    res.json({ ok: true });
  }));
}
