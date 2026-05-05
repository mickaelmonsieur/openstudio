import { streamDatabaseExport } from '../services/db-export.js';

export function registerDatabaseToolsRoutes(app, getDatabaseConfig) {
  app.get('/api/database/export', async (_req, res) => {
    try {
      await streamDatabaseExport(getDatabaseConfig(), res);
    } catch (error) {
      if (!res.headersSent) {
        res.status(500).json({ error: error.message });
      }
    }
  });
}
