import { useEffect, useRef, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

export function StationsPage() {
  const [rows, setRows] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [modal, setModal] = useState(null); // { mode: 'create'|'edit', row: null|{} }
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState(null);
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => { loadStations(); }, []);

  async function loadStations() {
    setLoading(true);
    setError(null);
    try {
      const payload = await fetchJson('/api/stations');
      setRows(payload.rows || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function save(data) {
    setSaving(true);
    setFormError(null);
    try {
      const editing = modal.mode === 'edit';
      const url = editing ? `/api/stations/${modal.row.id}` : '/api/stations';
      await fetchJson(url, { method: editing ? 'PUT' : 'POST', body: JSON.stringify(data) });
      setModal(null);
      await loadStations();
    } catch (err) {
      setFormError(err.message);
    } finally {
      setSaving(false);
    }
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await fetchJson(`/api/stations/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await loadStations();
    } catch (err) {
      setDeleteTarget(null);
      setError(err.message);
    } finally {
      setDeleting(false);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">General</p>
          <h2>Stations</h2>
        </div>
        <button
          className="primary-button"
          type="button"
          onClick={() => { setFormError(null); setModal({ mode: 'create', row: null }); }}
        >
          Add
        </button>
      </header>

      {error ? <div className="table-error">{error}</div> : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <div className="data-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th style={{ width: '80px' }}>ID</th>
                <th style={{ width: '200px' }}>Name</th>
                <th>Library Path</th>
                <th className="actions-column">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr><td className="empty-cell" colSpan={4}>No stations.</td></tr>
              ) : (
                rows.map((row) => (
                  <tr key={row.id}>
                    <td>{row.id}</td>
                    <td>{row.name}</td>
                    <td><code className="path-cell">{row.library_path || '—'}</code></td>
                    <td className="row-actions">
                      <button
                        className="ghost-button"
                        type="button"
                        onClick={() => { setFormError(null); setModal({ mode: 'edit', row }); }}
                      >Edit</button>
                      <button
                        className="danger-button"
                        type="button"
                        onClick={() => { setError(null); setDeleteTarget(row); }}
                      >Delete</button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {modal ? (
        <StationFormModal
          error={formError}
          mode={modal.mode}
          row={modal.row}
          saving={saving}
          onClose={() => setModal(null)}
          onSubmit={save}
        />
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={deleting}
          message={`Delete station "${deleteTarget.name}"? The library folder on disk will not be removed.`}
          title="Delete Station"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function StationFormModal({ mode, row, error, saving, onClose, onSubmit }) {
  const [name, setName] = useState(row?.name ?? '');
  const [libraryPath, setLibraryPath] = useState(row?.library_path ?? '');
  const [pathTouched, setPathTouched] = useState(mode === 'edit');
  const debounceRef = useRef(null);

  useEffect(() => {
    if (pathTouched) return;

    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(async () => {
      if (!name.trim()) return;
      try {
        const payload = await fetchJson(`/api/stations/suggest-path?name=${encodeURIComponent(name)}`);
        setLibraryPath(payload.path || '');
      } catch {
        // silent
      }
    }, 250);

    return () => clearTimeout(debounceRef.current);
  }, [name, pathTouched]);

  function submit(event) {
    event.preventDefault();
    onSubmit({ name, library_path: libraryPath });
  }

  return (
    <div className="modal-backdrop">
      <section className="modal-panel" role="dialog" aria-modal="true">
        <header className="modal-header">
          <div>
            <p className="panel-kicker">{mode === 'create' ? 'Add' : 'Edit'}</p>
            <h2>Station</h2>
          </div>
          <button className="icon-button" onClick={onClose} type="button">×</button>
        </header>

        <form className="resource-form" onSubmit={submit}>
          <label>
            <span>Name</span>
            <input
              maxLength={64}
              required
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </label>

          <label>
            <span>Library Path</span>
            <input
              required
              type="text"
              value={libraryPath}
              onChange={(e) => { setPathTouched(true); setLibraryPath(e.target.value); }}
            />
          </label>

          {error ? <div className="form-error">{error}</div> : null}

          <div className="form-actions">
            <button className="ghost-button" onClick={onClose} type="button">Cancel</button>
            <button className="primary-button" disabled={saving} type="submit">
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) },
    ...options
  });
  if (response.status === 204) return {};
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
  return payload;
}
