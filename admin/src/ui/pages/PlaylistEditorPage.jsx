import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

const HOURS = Array.from({ length: 24 }, (_, hour) => hour);

function emptyForm() {
  return {
    track_id: '',
    track_label: '',
    cue_in: 0,
    cue_out: 0,
    stretch_rate: 1
  };
}

export function PlaylistEditorPage() {
  const today = dateInputValue(new Date());
  const [scheduledDate, setScheduledDate] = useState(today);
  const [scheduledHour, setScheduledHour] = useState(new Date().getHours());
  const [rows, setRows] = useState([]);
  const [timezone, setTimezone] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [modal, setModal] = useState(null);
  const [formData, setFormData] = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving] = useState(false);
  const [ordering, setOrdering] = useState(false);
  const [draggingId, setDraggingId] = useState(null);
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [trackSearch, setTrackSearch] = useState('');
  const [trackResults, setTrackResults] = useState([]);
  const [trackLoading, setTrackLoading] = useState(false);

  useEffect(() => { loadQueue(); }, [scheduledDate, scheduledHour]);

  async function loadQueue() {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams({
        scheduled_date: scheduledDate,
        scheduled_hour: String(scheduledHour)
      });
      const payload = await fetchJson(`/api/queue?${params.toString()}`);
      setRows(payload.rows || []);
      setTimezone(payload.timezone || '');
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  function openAdd(insertAfterRow = null) {
    setFormData(emptyForm());
    setTrackSearch('');
    setTrackResults([]);
    setFormError(null);
    setModal({ mode: 'add', insertAfterId: insertAfterRow?.id || null });
  }

  function openEdit(row) {
    setFormData({
      track_id: row.track_id ? String(row.track_id) : '',
      track_label: trackLabel(row),
      cue_in: row.cue_in ?? 0,
      cue_out: row.cue_out ?? 0,
      stretch_rate: row.stretch_rate ?? 1
    });
    setTrackSearch(trackLabel(row));
    setTrackResults([]);
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  function update(key, value) {
    setFormData((prev) => ({ ...prev, [key]: value }));
  }

  async function searchTracks() {
    const query = trackSearch.trim();
    if (!query) return;

    setTrackLoading(true);
    setFormError(null);
    try {
      const params = new URLSearchParams({ page: '1', limit: '25', q: query });
      const payload = await fetchJson(`/api/tracks?${params.toString()}`);
      setTrackResults(payload.rows || []);
    } catch (err) {
      setFormError(err.message);
    } finally {
      setTrackLoading(false);
    }
  }

  function selectTrack(track) {
    setFormData((prev) => ({
      ...prev,
      track_id: String(track.id),
      track_label: trackLabel(track),
      cue_in: track.cue_in ?? prev.cue_in,
      cue_out: track.cue_out ?? track.duration ?? prev.cue_out
    }));
    setTrackSearch(trackLabel(track));
    setTrackResults([]);
  }

  async function save(event) {
    event.preventDefault();
    setSaving(true);
    setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      const url = isEdit ? `/api/queue/${modal.row.id}` : '/api/queue';
      const payload = await fetchJson(url, {
        method: isEdit ? 'PUT' : 'POST',
        body: JSON.stringify({
          ...formData,
          scheduled_date: scheduledDate,
          scheduled_hour: scheduledHour,
          insert_after_id: modal?.insertAfterId || undefined
        })
      });
      setRows(payload.rows || []);
      setModal(null);
    } catch (err) {
      setFormError(err.message);
    } finally {
      setSaving(false);
    }
  }

  function handleDrop(targetRow) {
    if (ordering || !draggingId) return;
    if (targetRow.id === draggingId) return;

    const fromIndex = rows.findIndex((row) => row.id === draggingId);
    const toIndex = rows.findIndex((row) => row.id === targetRow.id);
    if (fromIndex < 0 || toIndex < 0) return;

    const nextRows = [...rows];
    const [moved] = nextRows.splice(fromIndex, 1);
    nextRows.splice(toIndex, 0, moved);
    saveOrder(nextRows);
  }

  async function moveRow(row, direction) {
    if (ordering) return;
    const fromIndex = rows.findIndex((entry) => entry.id === row.id);
    const toIndex = fromIndex + direction;
    if (fromIndex < 0 || toIndex < 0 || toIndex >= rows.length) return;

    const nextRows = [...rows];
    const [moved] = nextRows.splice(fromIndex, 1);
    nextRows.splice(toIndex, 0, moved);
    await saveOrder(nextRows);
  }

  async function saveOrder(nextRows) {
    const previous = rows;
    setRows(nextRows);
    setOrdering(true);
    setError(null);
    try {
      const payload = await fetchJson('/api/queue/hour/order', {
        method: 'PUT',
        body: JSON.stringify({
          scheduled_date: scheduledDate,
          scheduled_hour: scheduledHour,
          ids: nextRows.map((row) => row.id)
        })
      });
      setRows(payload.rows || nextRows);
    } catch (err) {
      setRows(previous);
      setError(err.message);
    } finally {
      setOrdering(false);
    }
  }

  async function confirmDelete() {
    setSaving(true);
    setError(null);
    try {
      const params = new URLSearchParams({
        scheduled_date: scheduledDate,
        scheduled_hour: String(scheduledHour)
      });
      await fetchJson(`/api/queue/${deleteTarget.id}?${params.toString()}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await loadQueue();
    } catch (err) {
      setError(err.message);
      setDeleteTarget(null);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Automation</p>
          <h2>Playlist Editor</h2>
        </div>
        <div className="header-actions">
          <label className="table-search">
            <span>From Date</span>
            <input type="date" value={scheduledDate} onChange={(event) => setScheduledDate(event.target.value)} />
          </label>
          <label className="table-search">
            <span>From Hour</span>
            <select value={scheduledHour} onChange={(event) => setScheduledHour(Number(event.target.value))}>
              {HOURS.map((hour) => (
                <option key={hour} value={hour}>{formatHour(hour)}</option>
              ))}
            </select>
          </label>
          <button className="primary-button" type="button" onClick={() => openAdd()}>Add</button>
        </div>
      </header>

      <div className="filter-status">
        Showing: <strong>{scheduledDate} {formatHour(scheduledHour)} - {formatHour(scheduledHour + 1)}</strong>
        {timezone ? <> · Timezone: <strong>{timezone}</strong></> : null}
      </div>
      {error ? <div className="table-error">{error}</div> : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <div className="data-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th style={{ width: '105px' }}>Scheduled</th>
                <th style={{ width: '110px' }}>Type</th>
                <th style={{ width: '190px' }}>Artist</th>
                <th>Title</th>
                <th style={{ width: '90px' }}>Duration</th>
                <th style={{ width: '105px' }}>Play Duration</th>
                <th className="actions-column wide-actions">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr><td className="empty-cell" colSpan={7}>No playlist entries for this hour.</td></tr>
              ) : rows.map((row, index) => (
                <tr
                  key={row.id}
                  className={[
                    'draggable-row',
                    draggingId === row.id ? 'dragging-row' : ''
                  ].filter(Boolean).join(' ')}
                  style={row.track_type_color ? { backgroundColor: row.track_type_color } : undefined}
                  draggable={!ordering}
                  onDragEnd={() => setDraggingId(null)}
                  onDragOver={(event) => event.preventDefault()}
                  onDragStart={(event) => {
                    setDraggingId(row.id);
                    event.dataTransfer.effectAllowed = 'move';
                    event.dataTransfer.setData('text/plain', String(row.id));
                  }}
                  onDrop={(event) => {
                    event.preventDefault();
                    handleDrop(row);
                    setDraggingId(null);
                  }}
                >
                  <td>{row.scheduled_time}</td>
                  <td>{row.track_type_name || '—'}</td>
                  <td>{row.artist || '—'}</td>
                  <td>{row.title || '—'}</td>
                  <td>{formatSeconds(row.duration)}</td>
                  <td>{formatPlayDuration(row)}</td>
                  <td className="row-actions">
                    <button className="ghost-button table-icon-button" type="button" title="Add below" onClick={() => openAdd(row)}>
                      <i className="bi bi-plus-lg" aria-hidden="true" />
                    </button>
                    <button className="ghost-button table-icon-button" disabled={ordering || index === 0} type="button" title="Move up" onClick={() => moveRow(row, -1)}>
                      <i className="bi bi-arrow-up" aria-hidden="true" />
                    </button>
                    <button className="ghost-button table-icon-button" disabled={ordering || index === rows.length - 1} type="button" title="Move down" onClick={() => moveRow(row, 1)}>
                      <i className="bi bi-arrow-down" aria-hidden="true" />
                    </button>
                    <button className="ghost-button" type="button" onClick={() => openEdit(row)}>Edit</button>
                    <button className="danger-button" type="button" onClick={() => { setError(null); setDeleteTarget(row); }}>Delete</button>
                  </td>
                </tr>
              ))}
            </tbody>
            {rows.length > 0 ? (() => {
              const endTime = computeEndTime(rows, scheduledHour);
              return (
                <tfoot>
                  <tr>
                    <td style={{ color: endTime.complete ? '#16a34a' : '#dc2626', fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>
                      {endTime.time}
                    </td>
                    <td colSpan={6} style={{ color: endTime.complete ? '#16a34a' : '#dc2626', fontSize: '0.8em' }}>
                      {endTime.complete ? '✓ Hour complete' : '✗ Hour incomplete'}
                    </td>
                  </tr>
                </tfoot>
              );
            })() : null}
          </table>
        </div>
      )}

      {modal ? (
        <div className="modal-backdrop">
          <section className="modal-panel" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal.mode === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Playlist Entry</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>

            <form className="resource-form" onSubmit={save}>
              <label>
                <span>Track</span>
                <div className="track-picker">
                  <input type="search" value={trackSearch} onChange={(event) => setTrackSearch(event.target.value)} />
                  <button className="ghost-button" disabled={trackLoading} type="button" onClick={searchTracks}>
                    {trackLoading ? 'Searching...' : 'Search'}
                  </button>
                </div>
              </label>
              {formData.track_label ? <div className="selected-track">Selected: <strong>{formData.track_label}</strong></div> : null}
              {trackResults.length > 0 ? (
                <div className="track-results">
                  {trackResults.map((track) => (
                    <button key={track.id} type="button" onClick={() => selectTrack(track)}>
                      {trackLabel(track)}
                    </button>
                  ))}
                </div>
              ) : null}

              <div className="form-row">
                <NumberField label="Cue In" step="0.001" value={formData.cue_in} onChange={(value) => update('cue_in', value)} />
                <NumberField label="Cue Out" step="0.001" value={formData.cue_out} onChange={(value) => update('cue_out', value)} />
              </div>
              <NumberField label="Stretch Rate" min="0.001" step="0.001" value={formData.stretch_rate} onChange={(value) => update('stretch_rate', value)} />

              {formError ? <div className="form-error">{formError}</div> : null}

              <div className="form-actions">
                <button className="ghost-button" type="button" onClick={() => setModal(null)}>Cancel</button>
                <button className="primary-button" disabled={saving} type="submit">
                  {saving ? 'Saving...' : 'Save'}
                </button>
              </div>
            </form>
          </section>
        </div>
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={saving}
          message={`Delete "${trackLabel(deleteTarget)}" from this hour? Scheduled times will be recalculated.`}
          title="Delete Playlist Entry"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function NumberField({ label, value, onChange, min = '0', max, step }) {
  return (
    <label>
      <span>{label}</span>
      <input max={max} min={min} step={step} type="number" value={value} onChange={(event) => onChange(Number(event.target.value))} />
    </label>
  );
}

function trackLabel(track) {
  return [track.artist, track.title].filter(Boolean).join(' - ') || `Track #${track.track_id || track.id}`;
}

function dateInputValue(date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function pad(value) {
  return String(value).padStart(2, '0');
}

function formatHour(hour) {
  return `${pad(hour % 24)}:00`;
}

function formatPlayDuration(row) {
  const duration = Math.max(0, Number(row.cue_out || 0) - Number(row.cue_in || 0));
  return formatSeconds(duration);
}

function computeEndTime(rows, scheduledHour) {
  const last = rows[rows.length - 1];
  const [h, m, s] = last.scheduled_time.split(':').map(Number);
  const scheduledSeconds = h * 3600 + m * 60 + s;

  const cueIn = Number(last.cue_in ?? 0);
  const cueOut = Number(last.cue_out ?? 0);
  const stretchRate = Number(last.stretch_rate ?? 1);
  const playDuration = Math.max(0, (cueOut - cueIn) / stretchRate);

  const endSeconds = scheduledSeconds + playDuration;
  const offsetInHour = endSeconds - scheduledHour * 3600;

  const eh = Math.floor(endSeconds / 3600) % 24;
  const em = Math.floor((endSeconds % 3600) / 60);
  const es = Math.floor(endSeconds % 60);

  return {
    time: `${pad(eh)}:${pad(em)}:${pad(es)}`,
    complete: offsetInHour >= 3599
  };
}

function formatSeconds(value) {
  const duration = Number(value || 0);
  if (!Number.isFinite(duration)) return '—';

  const totalSeconds = Math.round(duration);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${pad(minutes)}:${pad(seconds)}`;
  }

  return `${minutes}:${pad(seconds)}`;
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }
  });
  if (response.status === 204) return {};
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
  return payload;
}
