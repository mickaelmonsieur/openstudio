import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const COLS = [
  { key: 'id',          label: 'ID',      width: '60px' },
  { key: 'name',        label: 'Name' },
  { key: 'sector_name', label: 'Sector',  width: '140px' },
  { key: 'vat_number',  label: 'VAT',     width: '120px' },
  { key: 'active',      label: 'Active',  width: '70px' },
  { key: 'client_since',label: 'Since',   width: '100px' }
];

const LIMIT = 50;

function emptyForm() {
  return { name: '', sector_id: '', address: '', vat_number: '', notes: '', active: true, client_since: '' };
}

export function AdvertisersPage() {
  const [rows, setRows]         = useState([]);
  const [total, setTotal]       = useState(0);
  const [page, setPage]         = useState(1);
  const [searchInput, setSearchInput]   = useState('');
  const [searchQuery, setSearchQuery]   = useState('');
  const [sectors, setSectors]   = useState([]);
  const [loading, setLoading]   = useState(true);
  const [error, setError]       = useState(null);
  const [modal, setModal]       = useState(null);
  const [form, setForm]         = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving]     = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  useEffect(() => {
    fetchJson('/api/sectors').then((s) => setSectors(s.rows || [])).catch(() => {});
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => { setSearchQuery(searchInput.trim()); setPage(1); }, 250);
    return () => clearTimeout(timer);
  }, [searchInput]);

  useEffect(() => { load(); }, [page, searchQuery]);

  async function load() {
    setLoading(true); setError(null);
    try {
      const params = new URLSearchParams({ page: String(page), limit: String(LIMIT) });
      if (searchQuery) params.set('q', searchQuery);
      const payload = await fetchJson(`/api/advertisers?${params}`);
      setRows(payload.rows || []);
      setTotal(payload.total || 0);
    } catch (err) { setError(err.message); }
    finally { setLoading(false); }
  }

  function upd(key, value) { setForm((prev) => ({ ...prev, [key]: value })); }

  function openAdd() { setForm(emptyForm()); setFormError(null); setModal('add'); }

  function openEdit(row) {
    setForm({
      name:         row.name         || '',
      sector_id:    row.sector_id    ? String(row.sector_id) : '',
      address:      row.address      || '',
      vat_number:   row.vat_number   || '',
      notes:        row.notes        || '',
      active:       Boolean(row.active ?? true),
      client_since: row.client_since || ''
    });
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  async function save(event) {
    event.preventDefault();
    setSaving(true); setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      await fetchJson(isEdit ? `/api/advertisers/${modal.row.id}` : '/api/advertisers', {
        method: isEdit ? 'PUT' : 'POST',
        body: JSON.stringify(form)
      });
      setModal(null);
      await load();
    } catch (err) { setFormError(err.message); }
    finally { setSaving(false); }
  }

  async function confirmDelete() {
    setSaving(true); setError(null);
    try {
      await fetchJson(`/api/advertisers/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await load();
    } catch (err) { setError(err.message); setDeleteTarget(null); }
    finally { setSaving(false); }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div><p className="panel-kicker">Advertising</p><h2>Advertisers</h2></div>
        <div className="header-actions">
          <label className="table-search">
            <span>Search</span>
            <input type="search" value={searchInput} onChange={(e) => setSearchInput(e.target.value)} />
          </label>
          <span className="log-total">{total.toLocaleString()} advertisers</span>
          <button className="primary-button" type="button" onClick={openAdd}>Add</button>
        </div>
      </header>

      {error ? <div className="table-error">{error}</div> : null}
      {loading ? <div className="table-loading">Loading...</div> : (
        <DataTable columns={COLS} primaryKey="id" rows={rows}
          onEdit={openEdit}
          onDelete={(row) => { setError(null); setDeleteTarget(row); }}
        />
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
                <h2>Advertiser</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>
            <form className="resource-form" onSubmit={save}>
              <label><span>Name *</span>
                <input autoFocus required maxLength={255} value={form.name} onChange={(e) => upd('name', e.target.value)} />
              </label>
              <label><span>Sector</span>
                <select value={form.sector_id} onChange={(e) => upd('sector_id', e.target.value)}>
                  <option value="">— none —</option>
                  {sectors.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
              </label>
              <label><span>VAT Number</span>
                <input maxLength={32} value={form.vat_number} onChange={(e) => upd('vat_number', e.target.value)} />
              </label>
              <label><span>Client Since</span>
                <input type="date" value={form.client_since} onChange={(e) => upd('client_since', e.target.value)} />
              </label>
              <label><span>Address</span>
                <textarea rows={3} value={form.address} onChange={(e) => upd('address', e.target.value)} />
              </label>
              <label><span>Notes</span>
                <textarea rows={3} value={form.notes} onChange={(e) => upd('notes', e.target.value)} />
              </label>
              <label className="checkbox-field"><span>Active</span>
                <input type="checkbox" checked={form.active} onChange={(e) => upd('active', e.target.checked)} />
              </label>
              {formError ? <div className="form-error">{formError}</div> : null}
              <div className="form-actions">
                <button className="ghost-button" type="button" onClick={() => setModal(null)}>Cancel</button>
                <button className="primary-button" disabled={saving} type="submit">{saving ? 'Saving...' : 'Save'}</button>
              </div>
            </form>
          </section>
        </div>
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog busy={saving} message={`Delete "${deleteTarget.name}"?`} title="Delete Advertiser"
          onCancel={() => setDeleteTarget(null)} onConfirm={confirmDelete} />
      ) : null}
    </section>
  );
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }, ...options
  });
  if (response.status === 204) return {};
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
  return payload;
}
