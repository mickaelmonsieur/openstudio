import { BrowserWindow } from 'electron';

export async function htmlToPdf(html) {
  const win = new BrowserWindow({ show: false });
  await win.loadURL('data:text/html;charset=utf-8,' + encodeURIComponent(html));
  try {
    return await win.webContents.printToPDF({ printBackground: true, pageSize: 'A4' });
  } finally {
    win.destroy();
  }
}

export function generatePlaylistHtml(hourBlocks, timezone, stationName = '') {
  const pages = hourBlocks.map((block, idx) =>
    renderHourPage(block.rows, block.date, block.hour, timezone, stationName, idx === hourBlocks.length - 1)
  ).join('\n');

  return `<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<style>
  @page { size: A4 portrait; margin: 12mm; }
  * { box-sizing: border-box; }
  body { font-family: Arial, Helvetica, sans-serif; font-size: 9pt; color: #1a1a1a; margin: 0; }
  .page { break-after: page; }
  .last-page { break-after: auto; }
  .page-header { margin-bottom: 3mm; border-bottom: 1.5pt solid #2d3748; padding-bottom: 2mm; }
  .page-header-row { display: flex; align-items: baseline; justify-content: space-between; }
  .page-header h2 { margin: 0; font-size: 12pt; }
  .page-header p { margin: 1mm 0 0; font-size: 7.5pt; color: #555; }
  .page-brand { font-size: 8pt; color: #888; font-style: italic; white-space: nowrap; }
  table { width: 100%; border-collapse: collapse; font-size: 8pt; table-layout: fixed; }
  colgroup col.c-time   { width: 58pt; }
  colgroup col.c-type   { width: 68pt; }
  colgroup col.c-artist { width: 110pt; }
  colgroup col.c-title  { }
  colgroup col.c-dur    { width: 38pt; }
  colgroup col.c-play   { width: 38pt; }
  thead tr { background: #2d3748; color: #fff; }
  thead th { padding: 3px 5px; text-align: left; font-weight: 600; font-size: 7.5pt; white-space: nowrap; overflow: hidden; }
  thead th.right { text-align: right; }
  tbody tr { break-inside: avoid; }
  tbody td { padding: 2.5px 5px; border-bottom: 0.5pt solid #e2e8f0; vertical-align: middle; overflow: hidden; }
  .c-time { white-space: nowrap; font-variant-numeric: tabular-nums; font-family: monospace; }
  .c-type { font-size: 7pt; }
  .c-dur, .c-play { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .page-footer { margin-top: 2.5mm; font-size: 8pt; font-weight: bold; }
  .complete { color: #166534; }
  .incomplete { color: #991b1b; }
</style>
</head>
<body>
${pages}
</body>
</html>`;
}

function renderHourPage(rows, date, hour, timezone, stationName, isLast) {
  const endTime = computeEndTime(rows, hour);
  const rowsHtml = rows.length === 0
    ? '<tr><td colspan="6" style="color:#999;font-style:italic;padding:6px 5px">Aucune piste</td></tr>'
    : rows.map((row) => {
        const bg = row.track_type_color || null;
        const style = bg ? ` style="background-color:${esc(bg)};color:#fff"` : '';
        const cueIn = Number(row.cue_in ?? 0);
        const cueOut = Number(row.cue_out ?? 0);
        const trackDur = Number(row.duration ?? 0);
        const playDur = Math.max(0, cueOut - cueIn);
        return `<tr${style}>
    <td class="c-time">${esc(row.scheduled_time)}</td>
    <td class="c-type">${esc(row.track_type_name || '—')}</td>
    <td class="c-artist" style="white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${esc(row.artist || '—')}</td>
    <td style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(row.title || '—')}</td>
    <td class="c-dur">${fmtDur(trackDur)}</td>
    <td class="c-play">${fmtDur(playDur)}</td>
  </tr>`;
      }).join('\n');

  const statusClass = endTime.complete ? 'complete' : 'incomplete';
  const statusText = endTime.complete ? '✓ Heure complète' : '✗ Heure incomplète';
  const footer = rows.length > 0
    ? `<p class="page-footer ${statusClass}">${endTime.time} · ${statusText}</p>`
    : '';

  const stationPart = stationName ? ` — ${esc(stationName)}` : '';
  return `<div class="page${isLast ? ' last-page' : ''}">
  <div class="page-header">
    <div class="page-header-row">
      <h2>Playlist${stationPart} — ${esc(date)} &nbsp; ${pad(hour)}:00 – ${pad(hour)}:59</h2>
      <span class="page-brand">OpenStudio</span>
    </div>
    <p>${rows.length} piste${rows.length !== 1 ? 's' : ''} &nbsp;·&nbsp; ${esc(timezone)}</p>
  </div>
  <table>
    <colgroup>
      <col class="c-time"><col class="c-type"><col class="c-artist">
      <col class="c-title"><col class="c-dur"><col class="c-play">
    </colgroup>
    <thead><tr>
      <th class="c-time">Heure</th>
      <th class="c-type">Type</th>
      <th class="c-artist">Artiste</th>
      <th>Titre</th>
      <th class="c-dur right">Durée</th>
      <th class="c-play right">Play</th>
    </tr></thead>
    <tbody>${rowsHtml}</tbody>
  </table>
  ${footer}
</div>`;
}

function computeEndTime(rows, hour) {
  if (rows.length === 0) return { time: '—', complete: false };
  const last = rows[rows.length - 1];
  const [h, m, s] = last.scheduled_time.split(':').map(Number);
  const scheduledSec = h * 3600 + m * 60 + s;
  const cueIn = Number(last.cue_in ?? 0);
  const cueOut = Number(last.cue_out ?? 0);
  const stretchRate = Number(last.stretch_rate ?? 1);
  const playDur = Math.max(0, (cueOut - cueIn) / stretchRate);
  const endSec = scheduledSec + playDur;
  const offsetInHour = endSec - hour * 3600;
  const eh = Math.floor(endSec / 3600) % 24;
  const em = Math.floor((endSec % 3600) / 60);
  const es = Math.floor(endSec % 60);
  return { time: `${pad(eh)}:${pad(em)}:${pad(es)}`, complete: offsetInHour >= 3599 };
}

function fmtDur(seconds) {
  if (!Number.isFinite(seconds) || seconds < 0) return '—';
  const total = Math.round(seconds);
  return `${Math.floor(total / 60)}:${pad(total % 60)}`;
}

function pad(v) { return String(v).padStart(2, '0'); }

function esc(str) {
  return String(str ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
