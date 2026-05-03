import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const COLS = [
  { key: 'id',              label: 'ID',         width: '60px' },
  { key: 'advertiser_name', label: 'Advertiser', width: '160px' },
  { key: 'name',            label: 'Name' },
  { key: 'role',            label: 'Role',        width: '120px' },
  { key: 'phone',           label: 'Phone',       width: '120px' },
  { key: 'email',           label: 'Email',       width: '180px' },
  { key: 'primary_contact', label: 'Primary',     width: '75px' }
];

const LIMIT = 50;

function emptyForm() {
  return { advertiser_id: '', name: '', role: '', phone: '', email: '', primary_contact: false, notes: '' };
}

export function ContactsPage() {
  const [rows, setRows]               = useState([]);
  const [total, setTotal]             = useState(0);
  const [page, setPage]               = useState(1);
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [advertisers, setAdvertisers] = useState([]);
  const [loading, setLoading]         = useState(true);
  const [error, setError]             = useState(null);
  const [modal, setModal]             = useState(null);
  const [form, setForm]               = useState(emptyForm);
  const [formError, setFormError]     = useState(null);
  const [saving, setSaving]           = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  useEffect(() => {
    fetchJson('/api/advertisers').then((a) => setAdvertisers(a.rows || [])).catch(() => {});
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
      const payload = await fetchJson(`/api/contacts?${params}`);
      setRows(payload.rows || []);
      setTotal(payload.total || 0);
    } catch (err) { setError(err.message); }
    finally { setLoading(false); }
  }

  function upd(key, value) { setForm((prev) => ({ ...prev, [key]: value })); }

  function openAdd() { setForm(emptyForm()); setFormError(null); setModal('add'); }

  function openEdit(row) {
    setForm({
      advertiser_id:   row.advertiser_id ? String(row.advertiser_id) : '',
      name:            row.name            || '',
      role:            row.role            || '',
      phone:           row.phone           || '',
      email:           row.email           || '',
      primary_contact: Boolean(row.primary_contact),
      notes:           row.notes           || ''
    });
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  async function save(event) {
    event.preventDefault();
    setSaving(true); setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      await fetchJson(isEdit ? `/api/contacts/${modal.row.id}` : '/api/contacts', {
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
      await fetchJson(`/api/contacts/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await load();
    } catch (err) { setError(err.message); setDeleteTarget(null); }
    finally { setSaving(false); }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div><p className="panel-kicker">Advertising</p><h2>Contacts</h2></div>
        <div className="header-actions">
          <label className="table-search">
            <span>Search</span>
            <input type="search" value={searchInput} onChange={(e) => setSearchInput(e.target.value)} />
          </label>
          <span className="log-total">{total.toLocaleString()} contacts</span>
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
                <h2>Contact</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>
            <form className="resource-form" onSubmit={save}>
              <label><span>Advertiser *</span>
                <select required value={form.advertiser_id} onChange={(e) => upd('advertiser_id', e.target.value)}>
                  <option value="">— select —</option>
                  {advertisers.map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
                </select>
              </label>
              <label><span>Name *</span>
                <input autoFocus required maxLength={128} value={form.name} onChange={(e) => upd('name', e.target.value)} />
              </label>
              <label><span>Role</span>
                <input maxLength={64} value={form.role} onChange={(e) => upd('role', e.target.value)} />
              </label>
              <div className="form-row">
                <label><span>Phone</span>
                  <input type="tel" maxLength={32} value={form.phone} onChange={(e) => upd('phone', e.target.value)} />
                </label>
                <label><span>Email</span>
                  <input type="email" maxLength={128} value={form.email} onChange={(e) => upd('email', e.target.value)} />
                </label>
              </div>
              <label><span>Notes</span>
                <textarea rows={3} value={form.notes} onChange={(e) => upd('notes', e.target.value)} />
              </label>
              <label className="checkbox-field"><span>Primary Contact</span>
                <input type="checkbox" checked={form.primary_contact} onChange={(e) => upd('primary_contact', e.target.checked)} />
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
        <ConfirmDialog busy={saving} message={`Delete "${deleteTarget.name}"?`} title="Delete Contact"
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
