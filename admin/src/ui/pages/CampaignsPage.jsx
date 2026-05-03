import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const COLS = [
  { key: 'id',              label: 'ID',         width: '60px' },
  { key: 'advertiser_name', label: 'Advertiser', width: '160px' },
  { key: 'name',            label: 'Name' },
  { key: 'station_name',    label: 'Station',    width: '120px' },
  { key: 'active',          label: 'Active',     width: '65px' },
  { key: 'start_date',      label: 'Start',      width: '100px' },
  { key: 'end_date',        label: 'End',        width: '100px' },
  { key: 'broadcast_count', label: 'Aired',      width: '65px' },
  { key: 'total_broadcasts',label: 'Total',      width: '65px' }
];

const LIMIT = 50;

function emptyForm() {
  return { advertiser_id: '', name: '', station_id: '', total_broadcasts: 0, active: true, start_date: '', end_date: '' };
}

export function CampaignsPage() {
  const [rows, setRows]               = useState([]);
  const [total, setTotal]             = useState(0);
  const [page, setPage]               = useState(1);
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [advertisers, setAdvertisers] = useState([]);
  const [stations, setStations]       = useState([]);
  const [loading, setLoading]         = useState(true);
  const [error, setError]             = useState(null);
  const [modal, setModal]             = useState(null);
  const [form, setForm]               = useState(emptyForm);
  const [formError, setFormError]     = useState(null);
  const [saving, setSaving]           = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  useEffect(() => {
    Promise.all([fetchJson('/api/advertisers'), fetchJson('/api/stations')])
      .then(([a, s]) => { setAdvertisers(a.rows || []); setStations(s.rows || []); })
      .catch(() => {});
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
      const payload = await fetchJson(`/api/campaigns?${params}`);
      setRows(payload.rows || []);
      setTotal(payload.total || 0);
    } catch (err) { setError(err.message); }
    finally { setLoading(false); }
  }

  function upd(key, value) { setForm((prev) => ({ ...prev, [key]: value })); }

  function openAdd() { setForm(emptyForm()); setFormError(null); setModal('add'); }

  function openEdit(row) {
    setForm({
      advertiser_id:    row.advertiser_id ? String(row.advertiser_id) : '',
      name:             row.name             || '',
      station_id:       row.station_id    ? String(row.station_id)    : '',
      total_broadcasts: row.total_broadcasts ?? 0,
      active:           Boolean(row.active ?? true),
      start_date:       row.start_date       || '',
      end_date:         row.end_date         || ''
    });
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  async function save(event) {
    event.preventDefault();
    setSaving(true); setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      await fetchJson(isEdit ? `/api/campaigns/${modal.row.id}` : '/api/campaigns', {
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
      await fetchJson(`/api/campaigns/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await load();
    } catch (err) { setError(err.message); setDeleteTarget(null); }
    finally { setSaving(false); }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div><p className="panel-kicker">Advertising</p><h2>Campaigns</h2></div>
        <div className="header-actions">
          <label className="table-search">
            <span>Search</span>
            <input type="search" value={searchInput} onChange={(e) => setSearchInput(e.target.value)} />
          </label>
          <span className="log-total">{total.toLocaleString()} campaigns</span>
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
                <h2>Campaign</h2>
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
              <label><span>Name</span>
                <input autoFocus maxLength={255} value={form.name} onChange={(e) => upd('name', e.target.value)} />
              </label>
              <label><span>Station</span>
                <select value={form.station_id} onChange={(e) => upd('station_id', e.target.value)}>
                  <option value="">— any —</option>
                  {stations.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
              </label>
              <div className="form-row">
                <label><span>Start Date</span>
                  <input type="date" value={form.start_date} onChange={(e) => upd('start_date', e.target.value)} />
                </label>
                <label><span>End Date</span>
                  <input type="date" value={form.end_date} onChange={(e) => upd('end_date', e.target.value)} />
                </label>
              </div>
              <label><span>Total Broadcasts</span>
                <input type="number" min="0" value={form.total_broadcasts}
                  onChange={(e) => upd('total_broadcasts', Number(e.target.value))} />
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
        <ConfirmDialog busy={saving} message={`Delete "${deleteTarget.name}"?`} title="Delete Campaign"
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
