import fs from 'node:fs/promises';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { withDatabase } from '../db/client.js';
import { createTrack, trackExistsByPath } from '../repositories/tracks.js';
import { buildFlacTrackDraft } from './import-flac.js';
import { defaultLibraryRoot } from '../lib/platform.js';

const MAX_TREE_DEPTH = 4;
const MAX_MESSAGES = 40;
const jobs = new Map();

export function databaseRoot() {
  return defaultLibraryRoot();
}

export async function listDatabaseFolders(folderPath = '') {
  const fullPath = resolveDatabasePath(folderPath);
  const depth = pathDepth(fullPath);

  if (depth >= MAX_TREE_DEPTH) {
    return {
      folder: folderInfo(fullPath),
      children: []
    };
  }

  const entries = await fs.readdir(fullPath, { withFileTypes: true });
  const children = entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => folderInfo(path.join(fullPath, entry.name)))
    .sort((a, b) => a.name.localeCompare(b.name));

  return {
    folder: folderInfo(fullPath),
    children
  };
}

export function startFolderImport(databaseConfig, options) {
  const folderPath = resolveDatabasePath(options.folderPath);
  const job = {
    id: randomUUID(),
    status: 'queued',
    folderPath,
    includeSubfolders: Boolean(options.includeSubfolders),
    subcategory_id: options.subcategory_id,
    genre_id: options.genre_id,
    total: 0,
    processed: 0,
    created: 0,
    skipped: 0,
    errors: 0,
    current: '',
    messages: [],
    startedAt: new Date().toISOString(),
    finishedAt: null
  };

  jobs.set(job.id, job);
  setTimeout(() => {
    runFolderImportJob(databaseConfig, job).catch((error) => {
      job.status = 'failed';
      job.finishedAt = new Date().toISOString();
      addMessage(job, `Failed: ${error.message}`);
    });
  }, 0);

  return serializeJob(job);
}

export function getFolderImportJob(id) {
  const job = jobs.get(id);
  return job ? serializeJob(job) : null;
}

async function runFolderImportJob(databaseConfig, job) {
  job.status = 'scanning';
  addMessage(job, `Scanning ${job.folderPath}`);

  const files = await findFlacFiles(job.folderPath, {
    includeSubfolders: job.includeSubfolders,
    maxDepth: MAX_TREE_DEPTH - pathDepth(job.folderPath)
  });

  job.total = files.length;
  job.status = 'running';
  addMessage(job, `Found ${files.length} FLAC file${files.length === 1 ? '' : 's'}.`);

  await withDatabase(databaseConfig, async (db) => {
    for (const filePath of files) {
      job.current = filePath;
      job.processed += 1;

      const existing = await trackExistsByPath(db, filePath);
      if (existing) {
        job.skipped += 1;
        addMessage(job, `Skipped: ${filePath} already exists`);
        continue;
      }

      try {
        const draft = await buildFlacTrackDraft(db, filePath, path.basename(filePath), {
          inferGenre: false
        });

        await createTrack(db, {
          artist_id: draft.artist_id || null,
          genre_id: job.genre_id,
          title: draft.title,
          album: draft.album,
          year: draft.year || null,
          duration: draft.duration,
          sample_rate: draft.sample_rate,
          path: filePath,
          subcategory_id: job.subcategory_id,
          active: true
        });

        job.created += 1;
        addMessage(job, `Imported: ${filePath}`);
      } catch (error) {
        if (error.code === '23505') {
          job.skipped += 1;
          addMessage(job, `Skipped: ${filePath} already exists`);
          continue;
        }

        job.errors += 1;
        addMessage(job, `Error: ${filePath} - ${error.message}`);
      }
    }
  });

  job.status = 'completed';
  job.current = '';
  job.finishedAt = new Date().toISOString();
  addMessage(job, 'Completed.');
}

async function findFlacFiles(folderPath, { includeSubfolders, maxDepth }) {
  const files = [];

  async function walk(currentPath, depth) {
    const entries = await fs.readdir(currentPath, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(currentPath, entry.name);
      if (entry.isDirectory()) {
        if (includeSubfolders && depth < maxDepth) {
          await walk(fullPath, depth + 1);
        }
        continue;
      }

      if (entry.isFile() && path.extname(entry.name).toLowerCase() === '.flac') {
        files.push(fullPath);
      }
    }
  }

  await walk(folderPath, 0);
  return files.sort((a, b) => a.localeCompare(b));
}

function resolveDatabasePath(inputPath = '') {
  const root = path.resolve(DATABASE_DIR);
  const requested = String(inputPath || '').trim();
  const fullPath = requested ? path.resolve(requested) : root;

  if (fullPath !== root && !fullPath.startsWith(`${root}${path.sep}`)) {
    throw new Error('Folder must be inside the music database.');
  }

  return fullPath;
}

function folderInfo(folderPath) {
  const depth = pathDepth(folderPath);
  return {
    name: folderPath === path.resolve(DATABASE_DIR) ? 'Database' : path.basename(folderPath),
    path: folderPath,
    relativePath: path.relative(path.resolve(DATABASE_DIR), folderPath),
    depth,
    canOpen: depth < MAX_TREE_DEPTH
  };
}

function pathDepth(folderPath) {
  const relative = path.relative(path.resolve(DATABASE_DIR), folderPath);
  if (!relative) return 0;
  return relative.split(path.sep).filter(Boolean).length;
}

function addMessage(job, message) {
  job.messages.push({
    at: new Date().toISOString(),
    message
  });

  if (job.messages.length > MAX_MESSAGES) {
    job.messages.splice(0, job.messages.length - MAX_MESSAGES);
  }
}

function serializeJob(job) {
  return {
    ...job,
    messages: [...job.messages]
  };
}
