import os from 'node:os';
import path from 'node:path';

export function defaultLibraryRoot() {
  switch (os.platform()) {
    case 'win32':  return 'C:\\OpenStudio\\Library';
    case 'darwin': return '/Users/Shared/OpenStudio/Library';
    default:       return '/var/lib/openstudio/Library';
  }
}

export function stationSlug(name) {
  return String(name || '')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
}

export function suggestStationPath(name) {
  const slug = stationSlug(name);
  return slug ? path.join(defaultLibraryRoot(), slug) : defaultLibraryRoot();
}
