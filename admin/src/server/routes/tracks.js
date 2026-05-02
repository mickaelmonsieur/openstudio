import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import multer from 'multer';
import { withDatabase } from '../db/client.js';
import {
  countTracks,
  createTrack,
  deleteTrack,
  hasScheduledQueue,
  listGenres,
  listSubcategoriesWithCategory,
  listTracks,
  updateTrack
} from '../repositories/tracks.js';
import { importFlacTrack } from '../services/import-flac.js';
import {
  databaseRoot,
  getFolderImportJob,
  listDatabaseFolders,
  startFolderImport
} from '../services/folder-import.js';

const DEFAULT_LIMIT = 100;
const MAX_LIMIT = 500;
const uploadDir = path.join(os.tmpdir(), 'openstudio-admin-uploads');
fs.mkdirSync(uploadDir, { recursive: true });

const upload = multer({
  storage: multer.diskStorage({
    destination: uploadDir,
    filename(_req, file, callback) {
      const ext = path.extname(file.originalname).toLowerCase() || '.upload';
      callback(null, `${randomUUID()}${ext}`);
    }
  }),
  limits: {
    fileSize: 2 * 1024 * 1024 * 1024
  }
});

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1,             10) || 1);
  const limit = Math.min(MAX_LIMIT, Math.max(1, parseInt(query.limit || DEFAULT_LIMIT, 10) || DEFAULT_LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

function parseSearch(query) {
  return String(query.q || '').trim().slice(0, 120);
}

function validateTrack(data, options = {}) {
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

  const artist_id      = parseOptionalPositiveInteger(data?.artist_id);
  const genre_id       = parseOptionalPositiveInteger(data?.genre_id);
  const subcategory_id = parseOptionalPositiveInteger(data?.subcategory_id);
  const year           = parseOptionalYear(data?.year);
  const active         = Boolean(data?.active);

  if (artist_id === false) {
    return { ok: false, error: 'Artist is invalid.' };
  }

  if (subcategory_id === false) {
    return { ok: false, error: 'Category is invalid.' };
  }

  if (genre_id === false) {
    return { ok: false, error: 'Genre is invalid.' };
  }

  if (!genre_id) {
    return { ok: false, error: 'Genre is required.' };
  }

  if (!subcategory_id) {
    return { ok: false, error: 'Category is required.' };
  }

  if (year === false) {
    return { ok: false, error: 'Year is invalid.' };
  }

  if (!options.requirePath) {
    return { ok: true, value: { title, album, artist_id, genre_id, subcategory_id, year, active } };
  }

  const trackPath = String(data?.path || '').trim();
  if (!trackPath) {
    return { ok: false, error: 'Imported file path is missing.' };
  }

  const duration = parseOptionalFloat(data?.duration, 0);
  const parsedSampleRate = parseOptionalPositiveInteger(data?.sample_rate);

  if (duration === false) {
    return { ok: false, error: 'Duration is invalid.' };
  }

  if (parsedSampleRate === false) {
    return { ok: false, error: 'Sample rate is invalid.' };
  }

  const sample_rate = parsedSampleRate || 44100;

  return {
    ok: true,
    value: {
      title,
      album,
      artist_id,
      genre_id,
      subcategory_id,
      year,
      active,
      duration,
      sample_rate,
      path: trackPath
    }
  };
}

function parseOptionalPositiveInteger(value) {
  if (value === undefined || value === null || value === '') return null;
  const number = Number(value);
  return Number.isInteger(number) && number > 0 ? number : false;
}

function parseOptionalYear(value) {
  if (value === undefined || value === null || value === '') return null;
  const year = Number(value);
  return Number.isInteger(year) && year >= 1900 && year <= 2100 ? year : false;
}

function parseOptionalFloat(value, fallback) {
  if (value === undefined || value === null || value === '') return fallback;
  const number = Number(value);
  return Number.isFinite(number) && number >= 0 ? number : false;
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
    const search = parseSearch(req.query);

    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([
        countTracks(db, search),
        listTracks(db, { limit, offset, search })
      ])
    );

    res.json({ rows, total, page, limit, q: search });
  }));

  // Must be declared before /api/tracks/:id to avoid "options" being parsed as an id
  app.get('/api/tracks/options', asyncRoute(async (_req, res) => {
    const [genres, subcategories] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([listGenres(db), listSubcategoriesWithCategory(db)])
    );
    res.json({ genres, subcategories });
  }));

  app.get('/api/tracks/folders', asyncRoute(async (req, res) => {
    const payload = await listDatabaseFolders(req.query.path);
    res.json({
      root: databaseRoot(),
      ...payload
    });
  }));

  app.post('/api/tracks/folder-import', asyncRoute(async (req, res) => {
    const folderPath = String(req.body?.folderPath || '').trim();
    const genre_id = parseOptionalPositiveInteger(req.body?.genre_id);
    const subcategory_id = parseOptionalPositiveInteger(req.body?.subcategory_id);

    if (!folderPath) {
      res.status(400).json({ error: 'Folder is required.' });
      return;
    }

    if (!genre_id || genre_id === false) {
      res.status(400).json({ error: 'Genre is required.' });
      return;
    }

    if (!subcategory_id || subcategory_id === false) {
      res.status(400).json({ error: 'Category is required.' });
      return;
    }

    const job = startFolderImport(getDatabaseConfig(), {
      folderPath,
      genre_id,
      subcategory_id,
      includeSubfolders: req.body?.includeSubfolders !== false
    });

    res.status(202).json({ job });
  }));

  app.get('/api/tracks/folder-import/:jobId', asyncRoute(async (req, res) => {
    const job = getFolderImportJob(req.params.jobId);
    if (!job) {
      res.status(404).json({ error: 'Import job not found.' });
      return;
    }

    res.json({ job });
  }));

  app.post('/api/tracks/import-flac/preview', upload.single('file'), asyncRoute(async (req, res) => {
    try {
      const draft = await withDatabase(getDatabaseConfig(), (db) =>
        importFlacTrack(db, req.file)
      );

      res.json({ draft });
    } catch (error) {
      if (req.file?.path) {
        fs.promises.unlink(req.file.path).catch(() => {});
      }
      res.status(400).json({ error: error.message });
    }
  }));

  app.post('/api/tracks', asyncRoute(async (req, res) => {
    const track = validateTrack(req.body, { requirePath: true });
    if (!track.ok) {
      res.status(400).json({ error: track.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createTrack(db, track.value)
    );

    res.status(201).json({ row });
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
