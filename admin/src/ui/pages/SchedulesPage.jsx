import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

const DAYS = [
  { key: 'monday',    label: 'Mon' },
  { key: 'tuesday',   label: 'Tue' },
  { key: 'wednesday', label: 'Wed' },
  { key: 'thursday',  label: 'Thu' },
  { key: 'friday',    label: 'Fri' },
  { key: 'saturday',  label: 'Sat' },
  { key: 'sunday',    label: 'Sun' }
];

const HOURS = Array.from({ length: 24 }, (_, i) => i);

function emptyForm() {
  const days = {};
  for (const d of DAYS) days[d.key] = false;
  return { template_id: '', from_hour: 0, to_hour: 23, ...days };
}

export function SchedulesPage() {
  const [rows, setRows]           = useState([]);
  const [templates, setTemplates] = useState([]);
  const [loading, setLoading]     = useState(true);
  const [error, setError]         = useState(null);
  const [modal, setModal]         = useState(null);
  const [formData, setFormData]   = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving]       = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  useEffect(() => { load(); }, []);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const [schedPayload, optPayload] = await Promise.all([
        fetchJson('/api/schedules'),
        fetchJson('/api/schedules/options')
      ]);
      setRows(schedPayload.rows || []);
      setTemplates(optPayload.templates || []);
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
      template_id: String(row.template_id ?? ''),
      from_hour:   row.from_hour ?? 0,
      to_hour:     row.to_hour   ?? 23,
      monday:    Boolean(row.monday),
      tuesday:   Boolean(row.tuesday),
      wednesday: Boolean(row.wednesday),
      thursday:  Boolean(row.thursday),
      friday:    Boolean(row.friday),
      saturday:  Boolean(row.saturday),
      sunday:    Boolean(row.sunday)
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
      const url    = isEdit ? `/api/schedules/${modal.row.id}` : '/api/schedules';
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
      await fetchJson(`/api/schedules/${deleteTarget.id}`, { method: 'DELETE' });
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
          <h2>Schedules</h2>
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
                <th style={{ width: '70px' }}>ID</th>
                <th style={{ width: '220px' }}>Template</th>
                <th style={{ width: '160px' }}>Hours</th>
                <th>Days</th>
                <th className="actions-column">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr><td className="empty-cell" colSpan={5}>No schedules.</td></tr>
              ) : rows.map((row) => (
                <tr key={row.id}>
                  <td>{row.id}</td>
                  <td>{row.template_name || '—'}</td>
                  <td>{formatHour(row.from_hour)} – {formatHour(row.to_hour)}</td>
                  <td><DayBadges row={row} /></td>
                  <td className="row-actions">
                    <button className="ghost-button" type="button" onClick={() => openEdit(row)}>Edit</button>
                    <button className="danger-button" type="button" onClick={() => { setError(null); setDeleteTarget(row); }}>Delete</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {modal ? (
        <div className="modal-backdrop">
          <section className="modal-panel" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Schedule</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>

            <form className="resource-form" onSubmit={save}>
              <label>
                <span>Template</span>
                <select
                  required
                  value={formData.template_id}
                  onChange={(e) => update('template_id', e.target.value)}
                >
                  <option value="">— select —</option>
                  {templates.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name}{t.format_name ? ` — ${t.format_name}` : ''}{t.category_name ? ` / ${t.category_name}` : ''}
                    </option>
                  ))}
                </select>
              </label>

              <div className="form-row">
                <label>
                  <span>From</span>
                  <select
                    value={formData.from_hour}
                    onChange={(e) => update('from_hour', Number(e.target.value))}
                  >
                    {HOURS.map((h) => (
                      <option key={h} value={h}>{formatHour(h)}</option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>To</span>
                  <select
                    value={formData.to_hour}
                    onChange={(e) => update('to_hour', Number(e.target.value))}
                  >
                    {HOURS.map((h) => (
                      <option key={h} value={h}>{formatHour(h)}</option>
                    ))}
                  </select>
                </label>
              </div>

              <div className="day-picker">
                <span>Days</span>
                <div className="day-picker-row">
                  {DAYS.map((d) => (
                    <label key={d.key} className="day-toggle">
                      <input
                        checked={Boolean(formData[d.key])}
                        type="checkbox"
                        onChange={(e) => update(d.key, e.target.checked)}
                      />
                      <span>{d.label}</span>
                    </label>
                  ))}
                </div>
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
          message={`Delete this schedule (${formatHour(deleteTarget.from_hour)} – ${formatHour(deleteTarget.to_hour)})?`}
          title="Delete Schedule"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function DayBadges({ row }) {
  return (
    <span className="day-badges">
      {DAYS.map((d) => (
        <span key={d.key} className={`day-badge ${row[d.key] ? 'active' : ''}`}>
          {d.label}
        </span>
      ))}
    </span>
  );
}

function formatHour(h) {
  return `${String(h ?? 0).padStart(2, '0')}:00`;
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
