import fs from 'node:fs/promises';
import path from 'node:path';
import { parseFile } from 'music-metadata';
import { findOrCreateArtist } from '../repositories/artists.js';
import { findGenreByName } from '../repositories/tracks.js';
import { defaultLibraryRoot } from '../lib/platform.js';

export async function importFlacTrack(db, file, libraryRoot = '') {
  if (!file?.path || !file?.originalname) {
    throw new Error('No file was uploaded.');
  }

  // multer decodes the multipart filename as Latin-1; re-interpret the bytes as UTF-8
  const originalname = Buffer.from(file.originalname, 'latin1').toString('utf8');

  if (path.extname(originalname).toLowerCase() !== '.flac') {
    throw new Error('Only .flac files can be imported.');
  }

  const draft = await buildFlacTrackDraft(db, file.path, originalname);
  const importedPath = await copyIntoDatabase(file.path, originalname, libraryRoot);

  return {
    ...draft,
    path: importedPath
  };
}

export async function buildFlacTrackDraft(db, filePath, displayName, options = {}) {
  const metadata = await parseFile(filePath);
  assertFlacMetadata(metadata);

  const common = metadata.common || {};
  const format = metadata.format || {};

  const artistName = firstText(common.artist || common.artists?.[0]);
  const genreName = firstText(common.genre);
  const artist = artistName ? await findOrCreateArtist(db, artistName) : null;
  const genre = options.inferGenre === false
    ? null
    : (genreName ? await findGenreByName(db, genreName) : null);
  const title = firstText(common.title) || stripExtension(displayName);
  const album = firstText(common.album);
  const year = parseYear(common.year || common.date);

  return {
    artist_id: artist?.id || '',
    artist_name: artist?.name || '',
    genre_id: genre?.id || '',
    genre_name: genre?.name || '',
    title: title.slice(0, 64),
    album: album.slice(0, 64),
    year: year || '',
    duration: Number(format.duration || 0),
    sample_rate: Number(format.sampleRate || 44100),
    path: filePath,
    subcategory_id: '',
    active: true
  };
}

function assertFlacMetadata(metadata) {
  const format = metadata?.format || {};
  if (format.container !== 'FLAC' && format.codec !== 'FLAC') {
    throw new Error(
      `The selected file is not a valid FLAC file. Detected container=${format.container || 'unknown'}, codec=${format.codec || 'unknown'}.`
    );
  }
}

async function copyIntoDatabase(sourcePath, originalName, libraryRoot = '') {
  const DATABASE_DIR = libraryRoot || defaultLibraryRoot();
  await fs.mkdir(DATABASE_DIR, { recursive: true });
  const destination = await uniqueDestination(originalName, DATABASE_DIR);
  await fs.copyFile(sourcePath, destination);
  await fs.unlink(sourcePath).catch(() => {});
  return destination;
}

async function uniqueDestination(originalName, libraryRoot = '') {
  const DATABASE_DIR = libraryRoot || defaultLibraryRoot();
  const parsed = path.parse(safeFileName(originalName));
  const base = parsed.name || 'track';
  const ext = parsed.ext || '.flac';

  for (let index = 0; index < 10_000; index += 1) {
    const suffix = index === 0 ? '' : `-${index + 1}`;
    const candidate = path.join(DATABASE_DIR, `${base}${suffix}${ext}`);
    try {
      await fs.access(candidate);
    } catch {
      return candidate;
    }
  }

  throw new Error('Unable to find a free destination filename.');
}

function safeFileName(name) {
  // Replace slashes before basename so "AC/DC" is not split into a directory
  const bare = path.basename(name.replace(/\//g, '_'));
  const extIndex = bare.lastIndexOf('.');
  const rawBase = extIndex > 0 ? bare.slice(0, extIndex) : bare;
  const ext = (extIndex > 0 ? bare.slice(extIndex) : '.flac').toLowerCase();

  const base = rawBase
    // Ligatures first (NFD won't split these)
    .replace(/œ/g, 'oe').replace(/Œ/g, 'Oe')
    .replace(/æ/g, 'ae').replace(/Æ/g, 'Ae')
    .replace(/ß/g, 'ss')
    // Strip diacritics (é→e, ü→u, ñ→n…)
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    // Drop any remaining non-ASCII
    .replace(/[^\x00-\x7F]/g, '')
    // Spaces and slashes → underscore
    .replace(/[\s/]+/g, '_')
    // Keep only safe chars
    .replace(/[^a-zA-Z0-9_\-]/g, '')
    // Cleanup
    .replace(/_{2,}/g, '_')
    .replace(/^_+|_+$/g, '')
    || 'track';

  return `${base}${ext}`;
}

function stripExtension(name) {
  return path.parse(name).name;
}

function firstText(value) {
  if (Array.isArray(value)) {
    return firstText(value[0]);
  }

  return String(value || '').trim();
}

function parseYear(value) {
  const match = String(value || '').match(/\d{4}/);
  return match ? Number(match[0]) : null;
}
