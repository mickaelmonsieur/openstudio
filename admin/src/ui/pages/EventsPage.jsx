import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

function emptyForm() {
  return {
    hour: 0,
    minute: 0,
    second: 0,
    template_id: '',
    priority: 0,
    duration: 0
  };
}

const LIMIT = 50;

export function EventsPage() {
  const [rows, setRows] = useState([]);
  const [templates, setTemplates] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [modal, setModal] = useState(null);
  const [formData, setFormData] = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving] = useState(false);
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
    setFormData({
      hour: row.hour ?? 0,
      minute: row.minute ?? 0,
      second: row.second ?? 0,
      template_id: row.template_id ? String(row.template_id) : '',
      priority: row.priority ?? 0,
      duration: row.duration ?? 0
    });
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  function update(key, value) {
    setFormData((prev) => ({ ...prev, [key]: value }));
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
                <th style={{ width: '120px' }}>Time</th>
                <th>Template</th>
                <th style={{ width: '100px' }}>Priority</th>
                <th style={{ width: '110px' }}>Duration</th>
                <th className="actions-column">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr><td className="empty-cell" colSpan={5}>No events.</td></tr>
              ) : rows.map((row) => (
                <tr key={row.id}>
                  <td>{formatTime(row)}</td>
                  <td>{row.template_name || '—'}</td>
                  <td>{row.priority ?? 0}</td>
                  <td>{formatDuration(row.duration)}</td>
                  <td className="row-actions">
                    <button aria-label="Edit" className="ghost-button table-icon-button" title="Edit" type="button" onClick={() => openEdit(row)}><i aria-hidden="true" className="bi bi-pencil" /></button>
                    <button aria-label="Delete" className="danger-button table-icon-button" title="Delete" type="button" onClick={() => { setError(null); setDeleteTarget(row); }}><i aria-hidden="true" className="bi bi-trash" /></button>
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
          <section className="modal-panel" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Event</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>

            <form className="resource-form" onSubmit={save}>
              <div className="form-row three-columns">
                <NumberField label="Hour" max={23} min={0} value={formData.hour} onChange={(value) => update('hour', value)} />
                <NumberField label="Minute" max={59} min={0} value={formData.minute} onChange={(value) => update('minute', value)} />
                <NumberField label="Second" max={59} min={0} value={formData.second} onChange={(value) => update('second', value)} />
              </div>

              <label>
                <span>Template</span>
                <select
                  value={formData.template_id}
                  onChange={(event) => update('template_id', event.target.value)}
                >
                  <option value="">— none —</option>
                  {templates.map((template) => (
                    <option key={template.id} value={template.id}>{template.name}</option>
                  ))}
                </select>
              </label>

              <div className="form-row">
                <NumberField label="Priority" max={32767} min={-32768} value={formData.priority} onChange={(value) => update('priority', value)} />
                <label>
                  <span>Duration (s)</span>
                  <input
                    min="0"
                    step="0.001"
                    type="number"
                    value={formData.duration}
                    onChange={(event) => update('duration', Number(event.target.value))}
                  />
                </label>
              </div>

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
          message={`Delete event at ${formatTime(deleteTarget)}?`}
          title="Delete Event"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function NumberField({ label, min, max, value, onChange }) {
  return (
    <label>
      <span>{label}</span>
      <input
        max={max}
        min={min}
        required
        type="number"
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function formatTime(row) {
  return `${pad(row.hour)}:${pad(row.minute)}:${pad(row.second)}`;
}

function pad(value) {
  return String(value ?? 0).padStart(2, '0');
}

function formatDuration(value) {
  const duration = Number(value || 0);
  return Number.isFinite(duration) ? `${duration}s` : '—';
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
