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

export function registerLibraryRoutes(app, getDatabaseConfig) {
  app.post('/api/library/rebase', asyncRoute(async (req, res) => {
    const stationId = Number(req.body?.station_id);
    const oldPath = String(req.body?.old_path || '').trim();
    const newPath = String(req.body?.new_path || '').trim();

    if (!Number.isInteger(stationId) || stationId <= 0) {
      res.status(400).json({ error: 'Invalid station_id.' });
      return;
    }
    if (!oldPath) {
      res.status(400).json({ error: 'old_path is required.' });
      return;
    }
    if (!newPath) {
      res.status(400).json({ error: 'new_path is required.' });
      return;
    }
    if (oldPath === newPath) {
      res.status(400).json({ error: 'old_path and new_path are identical.' });
      return;
    }

    const { tracksUpdated, stationUpdated } = await withDatabase(getDatabaseConfig(), async (db) => {
      const tracksResult = await db.query(
        `UPDATE tracks
         SET path = $2 || SUBSTR(path, LENGTH($1) + 1)
         WHERE path LIKE $1 || '%'`,
        [oldPath, newPath]
      );

      const stationResult = await db.query(
        `UPDATE stations
         SET library_path = $2
         WHERE id = $1 AND library_path = $3`,
        [stationId, newPath, oldPath]
      );

      return {
        tracksUpdated: tracksResult.rowCount,
        stationUpdated: stationResult.rowCount > 0
      };
    });

    res.json({ tracksUpdated, stationUpdated });
  }));
}
