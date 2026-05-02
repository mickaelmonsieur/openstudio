import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const candidates = [
  path.join(os.homedir(), 'Library/Application Support/openstudio-admin/launcher.log'),
  path.join(os.homedir(), 'Library/Application Support/OpenStudio Admin/launcher.log'),
  path.join(os.homedir(), 'AppData/Roaming/openstudio-admin/launcher.log')
];

for (const candidate of candidates) {
  if (fs.existsSync(candidate)) {
    console.log(candidate);
    console.log(fs.readFileSync(candidate, 'utf8'));
    process.exit(0);
  }
}

console.log('No launcher log found.');
for (const candidate of candidates) {
  console.log(candidate);
}
