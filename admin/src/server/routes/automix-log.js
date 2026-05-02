import { withDatabase } from '../db/client.js';
import { countAutomixLog, listAutomixLog } from '../repositories/automix-log.js';

const DEFAULT_LIMIT = 100;
const MAX_LIMIT = 500;

function parsePagination(query) {
  const page = Math.max(1, parseInt(query.page || 1, 10) || 1);
  const limit = Math.min(MAX_LIMIT, Math.max(1, parseInt(query.limit || DEFAULT_LIMIT, 10) || DEFAULT_LIMIT));
  return { page, limit, offset: (page - 1) * limit };
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

export function registerAutomixLogRoutes(app, getDatabaseConfig) {
  app.get('/api/automix-log', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);

    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([
        countAutomixLog(db),
        listAutomixLog(db, { limit, offset })
      ])
    );

    res.json({ rows, total, page, limit });
  }));
}
