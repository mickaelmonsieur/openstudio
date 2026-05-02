import { useEffect, useState } from 'react';
import { ConfirmDialog } from './ConfirmDialog.jsx';
import { CrudFormModal } from './CrudFormModal.jsx';
import { DataTable } from './DataTable.jsx';

export function CrudPage({ resource }) {
  const [rows, setRows] = useState([]);
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [modal, setModal] = useState(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState(null);
  const [deleteTarget, setDeleteTarget] = useState(null);

  useEffect(() => {
    loadRows();
  }, [resource.endpoint, searchQuery]);

  useEffect(() => {
    if (!resource.searchable) return undefined;

    const timer = setTimeout(() => {
      setSearchQuery(searchInput.trim());
    }, 250);

    return () => clearTimeout(timer);
  }, [resource.searchable, searchInput]);

  async function loadRows() {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      if (resource.searchable && searchQuery) params.set('q', searchQuery);
      const url = params.toString()
        ? `${resource.endpoint}?${params.toString()}`
        : resource.endpoint;
      const response = await fetch(url);
      const payload = await readJsonResponse(response);
      setRows(payload.rows || []);
    } catch (loadError) {
      setError(loadError.message);
    } finally {
      setLoading(false);
    }
  }

  async function saveRow(data) {
    const editing = modal?.mode === 'edit';
    const url = editing
      ? `${resource.endpoint}/${modal.row[resource.primaryKey]}`
      : resource.endpoint;

    setSaving(true);
    setFormError(null);
    try {
      const response = await fetch(url, {
        method: editing ? 'PUT' : 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
      });
      await readJsonResponse(response);
      setModal(null);
      await loadRows();
    } catch (saveError) {
      setFormError(saveError.message);
    } finally {
      setSaving(false);
    }
  }

  async function deleteRow() {
    if (!deleteTarget) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const response = await fetch(
        `${resource.endpoint}/${deleteTarget[resource.primaryKey]}`,
        { method: 'DELETE' }
      );
      if (!response.ok) {
        await readJsonResponse(response);
      }
      setDeleteTarget(null);
      await loadRows();
    } catch (deleteError) {
      setError(deleteError.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">CRUD</p>
          <h2>{resource.title}</h2>
        </div>
        <div className="header-actions">
          {resource.searchable ? (
            <label className="table-search">
              <span>Search</span>
              <input
                type="search"
                value={searchInput}
                onChange={(event) => setSearchInput(event.target.value)}
              />
            </label>
          ) : null}

          {resource.actions.add ? (
            <button
              className="primary-button"
              onClick={() => setModal({ mode: 'create', row: null })}
              type="button"
            >
              Add
            </button>
          ) : null}
        </div>
      </header>

      {error ? <div className="table-error">{error}</div> : null}

      {resource.searchable && searchQuery ? (
        <div className="filter-status">
          Search: <strong>{searchQuery}</strong>
          <button
            className="ghost-button"
            type="button"
            onClick={() => {
              setSearchInput('');
              setSearchQuery('');
            }}
          >
            Clear
          </button>
        </div>
      ) : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <DataTable
          columns={resource.columns}
          primaryKey={resource.primaryKey}
          rows={rows}
          onDelete={setDeleteTarget}
          onEdit={(row) => setModal({ mode: 'edit', row })}
        />
      )}

      {modal ? (
        <CrudFormModal
          error={formError}
          mode={modal.mode}
          resource={resource}
          row={modal.row}
          saving={saving}
          onClose={() => setModal(null)}
          onSubmit={saveRow}
        />
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={saving}
          message={`Delete "${deleteTarget.name || deleteTarget[resource.primaryKey]}"?`}
          title={`Delete ${resource.title}`}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={deleteRow}
        />
      ) : null}
    </section>
  );
}

async function readJsonResponse(response) {
  if (response.status === 204) {
    return {};
  }

  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(payload.error || `Request failed with status ${response.status}`);
  }

  return payload;
}
