import fs from 'node:fs/promises';
import { randomUUID } from 'node:crypto';
import { withDatabase } from '../db/client.js';

const MAX_MISSING = 500;
const jobs = new Map();

export function startFileScan(databaseConfig) {
  const job = {
    id: randomUUID(),
    status: 'queued',
    total: 0,
    processed: 0,
    ok: 0,
    missing: [],
    startedAt: new Date().toISOString(),
    finishedAt: null
  };

  jobs.set(job.id, job);
  setTimeout(() => {
    runFileScanJob(databaseConfig, job).catch((error) => {
      job.status = 'failed';
      job.finishedAt = new Date().toISOString();
      job.error = error.message;
    });
  }, 0);

  return serializeJob(job);
}

export function getFileScanJob(id) {
  const job = jobs.get(id);
  return job ? serializeJob(job) : null;
}

async function runFileScanJob(databaseConfig, job) {
  job.status = 'running';

  const tracks = await withDatabase(databaseConfig, async (db) => {
    const { rows } = await db.query(`
      SELECT t.id, t.path, t.title, COALESCE(a.name, '') AS artist
      FROM tracks t
      LEFT JOIN artists a ON a.id = t.artist_id
      WHERE t.path IS NOT NULL AND t.path <> ''
      ORDER BY t.id
    `);
    return rows;
  });

  job.total = tracks.length;

  for (const track of tracks) {
    const exists = await fileExists(track.path);
    job.processed += 1;

    if (exists) {
      job.ok += 1;
    } else if (job.missing.length < MAX_MISSING) {
      const label = [track.artist, track.title].filter(Boolean).join(' — ') || `Track #${track.id}`;
      job.missing.push({ id: track.id, path: track.path, label });
    }
  }

  job.status = 'completed';
  job.finishedAt = new Date().toISOString();
}

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

function serializeJob(job) {
  return {
    id: job.id,
    status: job.status,
    total: job.total,
    processed: job.processed,
    ok: job.ok,
    missing: job.missing,
    error: job.error || null,
    startedAt: job.startedAt,
    finishedAt: job.finishedAt
  };
}
