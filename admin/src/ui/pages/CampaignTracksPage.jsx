import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const PUB_CATEGORY_ID = 8;

const COLS = [
  { key: 'id',            label: 'ID',       width: '60px' },
  { key: 'campaign_name', label: 'Campaign', width: '200px' },
  { key: 'track_display', label: 'Track' },
  { key: 'position',      label: 'Pos.',     width: '60px' }
];

function emptyForm() {
  return { campaign_id: '', track_id: '', track_label: '', position: '' };
}

export function CampaignTracksPage() {
  const [rows, setRows]           = useState([]);
  const [campaigns, setCampaigns] = useState([]);
  const [loading, setLoading]     = useState(true);
  const [error, setError]         = useState(null);
  const [modal, setModal]         = useState(null);
  const [form, setForm]           = useState(emptyForm);
  const [formError, setFormError] = useState(null);
  const [saving, setSaving]       = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const [trackSearch, setTrackSearch]   = useState('');
  const [trackResults, setTrackResults] = useState([]);
  const [trackLoading, setTrackLoading] = useState(false);

  useEffect(() => {
    Promise.all([fetchJson('/api/campaign-tracks'), fetchJson('/api/campaign-tracks/options')])
      .then(([ct, opts]) => {
        setRows(ct.rows || []);
        setCampaigns(opts.campaigns || []);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  async function reload() {
    const payload = await fetchJson('/api/campaign-tracks');
    setRows(payload.rows || []);
  }

  function upd(key, value) { setForm((prev) => ({ ...prev, [key]: value })); }

  function openAdd() {
    setForm(emptyForm());
    setTrackSearch('');
    setTrackResults([]);
    setFormError(null);
    setModal('add');
  }

  function openEdit(row) {
    setForm({
      campaign_id:  row.campaign_id ? String(row.campaign_id) : '',
      track_id:     row.track_id    ? String(row.track_id)    : '',
      track_label:  row.track_display || '',
      position:     row.position    ?? ''
    });
    setTrackSearch(row.track_display || '');
    setTrackResults([]);
    setFormError(null);
    setModal({ mode: 'edit', row });
  }

  async function searchTracks() {
    const query = trackSearch.trim();
    if (!query) return;
    setTrackLoading(true);
    setFormError(null);
    try {
      const params = new URLSearchParams({ page: '1', limit: '25', q: query, category_id: String(PUB_CATEGORY_ID) });
      const payload = await fetchJson(`/api/tracks?${params.toString()}`);
      setTrackResults(payload.rows || []);
    } catch (err) {
      setFormError(err.message);
    } finally {
      setTrackLoading(false);
    }
  }

  function selectTrack(track) {
    const label = [track.artist, track.title].filter(Boolean).join(' — ') || `Track #${track.id}`;
    setForm((prev) => ({ ...prev, track_id: String(track.id), track_label: label }));
    setTrackSearch(label);
    setTrackResults([]);
  }

  async function save(event) {
    event.preventDefault();
    if (!form.track_id) { setFormError('Please select a track.'); return; }
    setSaving(true); setFormError(null);
    try {
      const isEdit = modal?.mode === 'edit';
      await fetchJson(isEdit ? `/api/campaign-tracks/${modal.row.id}` : '/api/campaign-tracks', {
        method: isEdit ? 'PUT' : 'POST',
        body: JSON.stringify({ campaign_id: form.campaign_id, track_id: form.track_id, position: form.position })
      });
      setModal(null);
      await reload();
    } catch (err) { setFormError(err.message); }
    finally { setSaving(false); }
  }

  async function confirmDelete() {
    setSaving(true); setError(null);
    try {
      await fetchJson(`/api/campaign-tracks/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await reload();
    } catch (err) { setError(err.message); setDeleteTarget(null); }
    finally { setSaving(false); }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div><p className="panel-kicker">Advertising</p><h2>Campaigns Tracks</h2></div>
        <button className="primary-button" type="button" onClick={openAdd}>Add</button>
      </header>

      {error ? <div className="table-error">{error}</div> : null}
      {loading ? <div className="table-loading">Loading...</div> : (
        <DataTable columns={COLS} primaryKey="id" rows={rows}
          onEdit={openEdit}
          onDelete={(row) => { setError(null); setDeleteTarget(row); }}
        />
      )}

      {modal ? (
        <div className="modal-backdrop">
          <section className="modal-panel" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Campaign Track</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>
            <form className="resource-form" onSubmit={save}>
              <label><span>Campaign *</span>
                <select required value={form.campaign_id} onChange={(e) => upd('campaign_id', e.target.value)}>
                  <option value="">— select —</option>
                  {campaigns.map((c) => <option key={c.id} value={c.id}>{c.name || `Campaign #${c.id}`}</option>)}
                </select>
              </label>

              <label><span>Track (Pub)</span>
                <div className="track-picker">
                  <input
                    type="search"
                    placeholder="Search in Pub category…"
                    value={trackSearch}
                    onChange={(e) => setTrackSearch(e.target.value)}
                    onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchTracks(); } }}
                  />
                  <button className="ghost-button" disabled={trackLoading} type="button" onClick={searchTracks}>
                    {trackLoading ? 'Searching…' : 'Search'}
                  </button>
                </div>
              </label>

              {form.track_label ? (
                <div className="selected-track">Selected: <strong>{form.track_label}</strong></div>
              ) : null}

              {trackResults.length > 0 ? (
                <div className="track-results">
                  {trackResults.map((track) => (
                    <button key={track.id} type="button" onClick={() => selectTrack(track)}>
                      {[track.artist, track.title].filter(Boolean).join(' — ') || `Track #${track.id}`}
                    </button>
                  ))}
                </div>
              ) : null}

              <label><span>Position {modal === 'add' ? '(auto si vide)' : ''}</span>
                <input type="number" min="1" value={form.position}
                  onChange={(e) => upd('position', e.target.value)} />
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
        <ConfirmDialog busy={saving}
          message={`Delete "${deleteTarget.track_display || deleteTarget.id}" from campaign?`}
          title="Delete Campaign Track"
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
