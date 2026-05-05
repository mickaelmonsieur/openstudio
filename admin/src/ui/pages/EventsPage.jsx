import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

const EVENT_TYPES = [
  { id: 1, label: 'One time Only',           short: 'One time' },
  { id: 2, label: 'Repeat by Day',           short: 'By Day' },
  { id: 3, label: 'Repeat by Day and Hour',  short: 'By Day & Hour' },
  { id: 4, label: 'Repeat by Date',          short: 'By Date' },
  { id: 5, label: 'Repeat by Date and Hour', short: 'By Date & Hour' }
];

const ACTION_TYPES = [
  { id: 1, label: 'Template' },
  { id: 2, label: 'Track' }
];

const DAYS = [
  { bit: 0, label: 'Mon' },
  { bit: 1, label: 'Tue' },
  { bit: 2, label: 'Wed' },
  { bit: 3, label: 'Thu' },
  { bit: 4, label: 'Fri' },
  { bit: 5, label: 'Sat' },
  { bit: 6, label: 'Sun' }
];

const HOURS_LIST = Array.from({ length: 24 }, (_, i) => i);

function getBit(mask, bit) { return ((mask >>> bit) & 1) === 1; }
function toggleBit(mask, bit) { return mask ^ (1 << bit); }

function emptyAction() {
  return { action_type: 1, template_id: '', track_id: '', track_label: '', track_search: '', track_results: [], track_loading: false };
}

function emptyForm() {
  return {
    event_type: 2,
    days_mask:  127,
    hours_mask: 0,
    event_date: '',
    hour:       0,
    minute:     0,
    second:     0,
    priority:   0,
    duration:   0,
    actions:    [emptyAction()]
  };
}

function rowToForm(row) {
  return {
    event_type:  row.event_type  ?? 2,
    days_mask:   row.days_mask   ?? 127,
    hours_mask:  row.hours_mask  ?? 0,
    event_date:  row.event_date  || '',
    hour:        row.hour        ?? 0,
    minute:      row.minute      ?? 0,
    second:      row.second      ?? 0,
    priority:    row.priority    ?? 0,
    duration:    row.duration    ?? 0,
    actions:     (row.actions || []).length > 0
      ? row.actions.map((a) => ({
          action_type:   a.action_type,
          template_id:   a.template_id ? String(a.template_id) : '',
          track_id:      a.track_id    ? String(a.track_id)    : '',
          track_label:   a.track_name  || '',
          track_search:  a.track_name  || '',
          track_results: [],
          track_loading: false
        }))
      : [emptyAction()]
  };
}

const LIMIT = 50;

export function EventsPage() {
  const [rows, setRows]         = useState([]);
  const [templates, setTemplates] = useState([]);
  const [total, setTotal]       = useState(0);
  const [page, setPage]         = useState(1);
  const [loading, setLoading]   = useState(true);
  const [error, setError]       = useState(null);
  const [modal, setModal]       = useState(null);
  const [formData, setFormData] = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving]     = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  useEffect(() => { load(); }, [page]); // eslint-disable-line react-hooks/exhaustive-deps

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams({ page: String(page), limit: String(LIMIT) });
      const [eventsPayload, optionsPayload] = await Promise.all([
        fetchJson(`/api/events?${params}`),
        fetchJson('/api/events/options')
      ]);
      setRows(eventsPayload.rows || []);
      setTotal(eventsPayload.total || 0);
      setTemplates(optionsPayload.templates || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  function openAdd() {
    setFormData(emptyForm());
    setFormError(null);
    setModal('add');
  }

  function openEdit(row) {
    setFormData(rowToForm(row));
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  function update(key, value) {
    setFormData((prev) => ({ ...prev, [key]: value }));
  }

  function updateType(newType) {
    setFormData((prev) => ({ ...prev, event_type: newType }));
  }

  function updateDayBit(bit) {
    setFormData((prev) => ({ ...prev, days_mask: toggleBit(prev.days_mask, bit) }));
  }

  function updateHourBit(hour) {
    setFormData((prev) => ({ ...prev, hours_mask: toggleBit(prev.hours_mask, hour) }));
  }

  function updateAction(index, key, value) {
    setFormData((prev) => {
      const actions = prev.actions.map((a, i) =>
        i === index ? { ...a, [key]: value } : a
      );
      return { ...prev, actions };
    });
  }

  function updateActionBatch(index, updates) {
    setFormData((prev) => {
      const actions = prev.actions.map((a, i) =>
        i === index ? { ...a, ...updates } : a
      );
      return { ...prev, actions };
    });
  }

  async function searchTracksForAction(index) {
    const query = formData.actions[index].track_search.trim();
    if (!query) return;
    updateActionBatch(index, { track_loading: true, track_results: [] });
    try {
      const params = new URLSearchParams({ page: '1', limit: '25', q: query });
      const payload = await fetchJson(`/api/tracks?${params}`);
      updateActionBatch(index, { track_results: payload.rows || [], track_loading: false });
    } catch (err) {
      updateAction(index, 'track_loading', false);
      setFormError(err.message);
    }
  }

  function selectTrackForAction(index, track) {
    const label = [track.artist, track.title].filter(Boolean).join(' — ') || `Track #${track.id}`;
    updateActionBatch(index, { track_id: String(track.id), track_label: label, track_search: label, track_results: [] });
  }

  function addAction() {
    setFormData((prev) => ({ ...prev, actions: [...prev.actions, emptyAction()] }));
  }

  function removeAction(index) {
    setFormData((prev) => ({
      ...prev,
      actions: prev.actions.filter((_, i) => i !== index)
    }));
  }

  async function save(event) {
    event.preventDefault();
    setSaving(true);
    setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      const url = isEdit ? `/api/events/${modal.row.id}` : '/api/events';
      await fetchJson(url, {
        method: isEdit ? 'PUT' : 'POST',
        body: JSON.stringify(formData)
      });
      setModal(null);
      await load();
    } catch (err) {
      setFormError(err.message);
    } finally {
      setSaving(false);
    }
  }

  async function confirmDelete() {
    setSaving(true);
    setError(null);
    try {
      await fetchJson(`/api/events/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await load();
    } catch (err) {
      setError(err.message);
      setDeleteTarget(null);
    } finally {
      setSaving(false);
    }
  }

  const et = formData.event_type;
  const showDate     = [1, 4, 5].includes(et);
  const showDays     = [2, 3].includes(et);
  const showHours    = [3, 5].includes(et);
  const hourDisabled = [3, 5].includes(et);
  const dateLabel    = [4, 5].includes(et) ? 'Date (year ignored)' : 'Date';

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Automation</p>
          <h2>Events</h2>
        </div>
        <div className="header-actions">
          <button className="primary-button" type="button" onClick={openAdd}>Add</button>
        </div>
      </header>

      {error ? <div className="table-error">{error}</div> : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <div className="data-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th style={{ width: '130px' }}>Type</th>
                <th>Trigger</th>
                <th>Actions</th>
                <th style={{ width: '80px' }}>Priority</th>
                <th style={{ width: '90px' }}>Duration</th>
                <th className="actions-column">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr><td className="empty-cell" colSpan={6}>No events.</td></tr>
              ) : rows.map((row) => (
                <tr key={row.id}>
                  <td style={{ fontSize: '0.8em', color: '#91a9b7' }}>
                    {EVENT_TYPES.find((t) => t.id === row.event_type)?.short ?? '—'}
                  </td>
                  <td style={{ fontFamily: 'monospace', fontSize: '0.85em' }}>
                    {formatTrigger(row)}
                  </td>
                  <td style={{ fontSize: '0.85em' }}>
                    {(row.actions || []).map((a, i) => (
                      <span key={i} className="action-badge">
                        {a.action_type === 2
                          ? (a.track_name || '—')
                          : (a.template_name || '—')}
                      </span>
                    ))}
                  </td>
                  <td>{row.priority ?? 0}</td>
                  <td>{formatDuration(row.duration)}</td>
                  <td className="row-actions">
                    <button aria-label="Edit" className="ghost-button table-icon-button" title="Edit" type="button" onClick={() => openEdit(row)}>
                      <i aria-hidden="true" className="bi bi-pencil" />
                    </button>
                    <button aria-label="Delete" className="danger-button table-icon-button" title="Delete" type="button" onClick={() => { setError(null); setDeleteTarget(row); }}>
                      <i aria-hidden="true" className="bi bi-trash" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="pagination">
        <button className="ghost-button" disabled={page <= 1} type="button" onClick={() => setPage((p) => p - 1)}>← Prev</button>
        <span className="pagination-info">Page {page} of {totalPages} — {total.toLocaleString()} total</span>
        <button className="ghost-button" disabled={page >= totalPages} type="button" onClick={() => setPage((p) => p + 1)}>Next →</button>
      </div>

      {modal ? (
        <div className="modal-backdrop">
          <section className="modal-panel event-modal" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Event</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>

            <form className="resource-form event-form-grid" onSubmit={save}>

              {/* ── Left column ── */}
              <div className="event-col-left">
                <label>
                  <span>Type</span>
                  <select value={et} onChange={(e) => updateType(Number(e.target.value))}>
                    {EVENT_TYPES.map((t) => (
                      <option key={t.id} value={t.id}>{t.label}</option>
                    ))}
                  </select>
                </label>

                <hr className="form-separator" />

                {/* Date — types 1, 4, 5 */}
                {showDate ? (
                  <label>
                    <span>{dateLabel}</span>
                    <input
                      required
                      type="date"
                      value={formData.event_date}
                      onChange={(e) => update('event_date', e.target.value)}
                    />
                  </label>
                ) : null}

                {/* Day checkboxes — types 2, 3 */}
                {showDays ? (
                  <div className="day-picker">
                    <span>Days</span>
                    <div className="day-picker-row">
                      {DAYS.map((d) => (
                        <label key={d.bit} className="day-toggle">
                          <input
                            type="checkbox"
                            checked={getBit(formData.days_mask, d.bit)}
                            onChange={() => updateDayBit(d.bit)}
                          />
                          <span>{d.label}</span>
                        </label>
                      ))}
                    </div>
                  </div>
                ) : null}

                {/* Hour / Minute / Second */}
                <div className="form-row three-columns">
                  <NumberField
                    label="Hour"
                    max={23} min={0}
                    value={formData.hour}
                    disabled={hourDisabled}
                    onChange={(value) => update('hour', value)}
                  />
                  <NumberField label="Minute" max={59} min={0} value={formData.minute} onChange={(value) => update('minute', value)} />
                  <NumberField label="Second" max={59} min={0} value={formData.second} onChange={(value) => update('second', value)} />
                </div>
              </div>

              {/* ── Right column ── */}
              <div className="event-col-right">
                {/* Hour checkboxes — types 3, 5 */}
                {showHours ? (
                  <>
                    <div className="hour-picker">
                      <span>Hours</span>
                      <div className="hour-picker-grid">
                        {HOURS_LIST.map((h) => (
                          <label key={h} className="hour-toggle">
                            <input
                              type="checkbox"
                              checked={getBit(formData.hours_mask, h)}
                              onChange={() => updateHourBit(h)}
                            />
                            <span>{h}h</span>
                          </label>
                        ))}
                      </div>
                    </div>
                    <hr className="form-separator" />
                  </>
                ) : null}

                {/* Actions */}
                <div className="event-actions-section">
                  <span className="event-actions-label">Actions</span>
                  {formData.actions.map((action, i) => (
                    <div key={i} className="event-action-row">
                      <select
                        value={action.action_type}
                        onChange={(e) => updateAction(i, 'action_type', Number(e.target.value))}
                      >
                        {ACTION_TYPES.map((t) => (
                          <option key={t.id} value={t.id}>{t.label}</option>
                        ))}
                      </select>
                      {action.action_type === 1 ? (
                        <select
                          value={action.template_id}
                          onChange={(e) => updateAction(i, 'template_id', e.target.value)}
                        >
                          <option value="">— select —</option>
                          {templates.map((t) => (
                            <option key={t.id} value={t.id}>{t.name}</option>
                          ))}
                        </select>
                      ) : (
                        <div className="track-picker-inline">
                          <div className="track-picker">
                            <input
                              type="search"
                              placeholder="Search tracks…"
                              value={action.track_search}
                              onChange={(e) => updateAction(i, 'track_search', e.target.value)}
                              onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchTracksForAction(i); } }}
                            />
                            <button className="ghost-button" disabled={action.track_loading} type="button" onClick={() => searchTracksForAction(i)}>
                              {action.track_loading ? 'Searching…' : 'Search'}
                            </button>
                          </div>
                          {action.track_label ? (
                            <div className="selected-track">Selected: <strong>{action.track_label}</strong></div>
                          ) : null}
                          {action.track_results.length > 0 ? (
                            <div className="track-results">
                              {action.track_results.map((track) => (
                                <button key={track.id} type="button" onClick={() => selectTrackForAction(i, track)}>
                                  {[track.artist, track.title].filter(Boolean).join(' — ') || `Track #${track.id}`}
                                </button>
                              ))}
                            </div>
                          ) : null}
                        </div>
                      )}
                      {formData.actions.length > 1 ? (
                        <button className="icon-button" type="button" title="Remove" onClick={() => removeAction(i)}>×</button>
                      ) : null}
                    </div>
                  ))}
                  <button className="ghost-button add-action-button" type="button" onClick={addAction}>
                    + Add action
                  </button>
                </div>

                <hr className="form-separator" />

                {/* Priority + Duration */}
                <div className="form-row">
                  <NumberField label="Priority" max={32767} min={-32768} value={formData.priority} onChange={(value) => update('priority', value)} />
                  <label>
                    <span>Duration (s)</span>
                    <input
                      min="0"
                      step="0.001"
                      type="number"
                      value={formData.duration}
                      onChange={(e) => update('duration', Number(e.target.value))}
                    />
                  </label>
                </div>
              </div>

              {formError ? <div className="form-error event-form-full">{formError}</div> : null}

              <div className="form-actions event-form-full">
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
          message="Delete this event?"
          title="Delete Event"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function NumberField({ label, min, max, value, disabled, onChange }) {
  return (
    <label>
      <span>{label}</span>
      <input
        disabled={disabled}
        max={max}
        min={min}
        required={!disabled}
        type="number"
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

// ── Formatting helpers ────────────────────────────────────────────────────────

function formatDays(mask) {
  const active = DAYS.filter((d) => getBit(mask, d.bit)).map((d) => d.label);
  if (active.length === 0) return '—';
  if (active.length === 7) return 'Every day';
  return active.join(' ');
}

function formatHoursMask(mask) {
  const active = HOURS_LIST.filter((h) => getBit(mask, h));
  if (active.length === 0) return '—';
  return active.map((h) => `${h}h`).join(' ');
}

function formatDateFull(dateStr) {
  if (!dateStr) return '—';
  return dateStr.slice(0, 10);
}

function formatDateDayMonth(dateStr) {
  if (!dateStr) return '—';
  const parts = dateStr.slice(0, 10).split('-');
  return `${parts[2]}/${parts[1]}`;
}

function formatTrigger(row) {
  const mm = pad(row.minute);
  const ss = pad(row.second);
  const hms = `${pad(row.hour)}:${mm}:${ss}`;

  switch (row.event_type) {
    case 1: return `${formatDateFull(row.event_date)}  ${hms}`;
    case 2: return `${formatDays(row.days_mask)}  ${hms}`;
    case 3: return `${formatDays(row.days_mask)}  ${formatHoursMask(row.hours_mask)}  :${mm}:${ss}`;
    case 4: return `${formatDateDayMonth(row.event_date)}  ${hms}`;
    case 5: return `${formatDateDayMonth(row.event_date)}  ${formatHoursMask(row.hours_mask)}  :${mm}:${ss}`;
    default: return hms;
  }
}

function pad(value) {
  return String(value ?? 0).padStart(2, '0');
}

function formatDuration(value) {
  const d = Number(value || 0);
  return Number.isFinite(d) ? `${d}s` : '—';
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
