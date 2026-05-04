import { getFileScanJob, startFileScan } from '../services/file-scanner.js';

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

export function registerFileScanRoutes(app, getDatabaseConfig) {
  app.post('/api/file-scan', asyncRoute(async (_req, res) => {
    const job = startFileScan(getDatabaseConfig());
    res.status(202).json({ job });
  }));

  app.get('/api/file-scan/:id', asyncRoute(async (req, res) => {
    const job = getFileScanJob(req.params.id);
    if (!job) { res.status(404).json({ error: 'Scan job not found.' }); return; }
    res.json({ job });
  }));
}
