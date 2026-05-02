import fs from 'node:fs/promises';
import path from 'node:path';
import { parseFile } from 'music-metadata';
import { findOrCreateArtist } from '../repositories/artists.js';
import { findGenreByName } from '../repositories/tracks.js';

const DATABASE_DIR = '/Users/mickael/Music/Database';

export async function importFlacTrack(db, file) {
  if (!file?.path || !file?.originalname) {
    throw new Error('No file was uploaded.');
  }

  if (path.extname(file.originalname).toLowerCase() !== '.flac') {
    throw new Error('Only .flac files can be imported.');
  }

  const metadata = await parseFile(file.path);
  assertFlacMetadata(metadata);

  const common = metadata.common || {};
  const format = metadata.format || {};

  const artistName = firstText(common.artist || common.artists?.[0]);
  const genreName = firstText(common.genre);
  const artist = artistName ? await findOrCreateArtist(db, artistName) : null;
  const genre = genreName ? await findGenreByName(db, genreName) : null;
  const title = firstText(common.title) || stripExtension(file.originalname);
  const album = firstText(common.album);
  const year = parseYear(common.year || common.date);
  const importedPath = await copyIntoDatabase(file.path, file.originalname);

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
    path: importedPath,
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

async function copyIntoDatabase(sourcePath, originalName) {
  await fs.mkdir(DATABASE_DIR, { recursive: true });
  const destination = await uniqueDestination(originalName);
  await fs.copyFile(sourcePath, destination);
  await fs.unlink(sourcePath).catch(() => {});
  return destination;
}

async function uniqueDestination(originalName) {
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
  return path.basename(name).replace(/[/:*?"<>|]/g, '-').trim() || 'track.flac';
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
