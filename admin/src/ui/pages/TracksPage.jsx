import { useEffect, useRef, useState } from 'react';
import { useStation } from '../StationContext.jsx';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { ScanFolderModal } from './ScanFolderModal.jsx';
import { TrackEditModal } from './TrackEditModal.jsx';

const LIMIT = 100;

export function TracksPage() {
  const { stationId } = useStation();
  const fileInputRef = useRef(null);
  const [rows, setRows] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const [artists, setArtists] = useState([]);
  const [genres, setGenres] = useState([]);
  const [subcategories, setSubcategories] = useState([]);

  const [editTrack, setEditTrack] = useState(null);
  const [importDraft, setImportDraft] = useState(null);
  const [scanFolderOpen, setScanFolderOpen] = useState(false);
  const [importing, setImporting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState(null);

  const [deleteTarget, setDeleteTarget] = useState(null);
  const [deleting, setDeleting] = useState(false);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  useEffect(() => {
    loadOptions();
  }, []);

  useEffect(() => {
    loadTracks();
  }, [page, searchQuery]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchQuery(searchInput.trim());
      setPage(1);
    }, 250);

    return () => clearTimeout(timer);
  }, [searchInput]);

  async function loadOptions() {
    try {
      const [artistsPayload, optionsPayload] = await Promise.all([
        fetchJson('/api/artists'),
        fetchJson('/api/tracks/options')
      ]);
      setArtists(artistsPayload.rows || []);
      setGenres(optionsPayload.genres || []);
      setSubcategories(optionsPayload.subcategories || []);
    } catch (err) {
      setError(err.message);
    }
  }

  async function loadTracks() {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams({
        page: String(page),
        limit: String(LIMIT)
      });
      if (searchQuery) params.set('q', searchQuery);

      const payload = await fetchJson(`/api/tracks?${params.toString()}`);
      setRows(payload.rows || []);
      setTotal(payload.total || 0);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function saveTrack(data) {
    setSaving(true);
    setFormError(null);
    try {
      await fetchJson(`/api/tracks/${editTrack.id}`, {
        method: 'PUT',
        body: JSON.stringify(data)
      });
      setEditTrack(null);
      await loadTracks();
    } catch (err) {
      setFormError(err.message);
    } finally {
      setSaving(false);
    }
  }

  async function importFile(file) {
    if (!file) return;

    setImporting(true);
    setFormError(null);
    setError(null);

    try {
      const body = new FormData();
      body.append('file', file);
      if (stationId) body.append('station_id', stationId);

      const payload = await fetchJson('/api/tracks/import-flac/preview', {
        method: 'POST',
        body
      });

      await loadOptions();
      setImportDraft(payload.draft);
    } catch (err) {
      setError(err.message);
    } finally {
      setImporting(false);
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  }

  async function saveImportedTrack(data) {
    setSaving(true);
    setFormError(null);

    try {
      await fetchJson('/api/tracks', {
        method: 'POST',
        body: JSON.stringify(data)
      });
      setImportDraft(null);
      await loadOptions();
      if (page !== 1) {
        setPage(1);
      } else {
        await loadTracks();
      }
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
      await fetchJson(`/api/tracks/${deleteTarget.id}`, { method: 'DELETE' });
      setDeleteTarget(null);
      await loadTracks();
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
          <p className="panel-kicker">Library</p>
          <h2>Tracks</h2>
        </div>
        <div className="header-actions">
          <label className="table-search">
            <span>Search</span>
            <input
              type="search"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
            />
          </label>
          <input
            ref={fileInputRef}
            accept=".flac,audio/flac"
            className="hidden-file-input"
            type="file"
            onChange={(event) => importFile(event.target.files?.[0])}
          />
          <button
            className="primary-button"
            disabled={importing}
            type="button"
            onClick={() => fileInputRef.current?.click()}
          >
            {importing ? 'Importing...' : 'Add File'}
          </button>
          <button
            className="ghost-button"
            type="button"
            onClick={() => setScanFolderOpen(true)}
          >
            Scan Folder
          </button>
          <span className="log-total">{total.toLocaleString()} tracks</span>
        </div>
      </header>

      {error ? <div className="table-error">{error}</div> : null}

      {searchQuery ? (
        <div className="filter-status">
          Search: <strong>{searchQuery}</strong>
          <button
            className="ghost-button"
            type="button"
            onClick={() => {
              setSearchInput('');
              setSearchQuery('');
              setPage(1);
            }}
          >
            Clear
          </button>
        </div>
      ) : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <>
          <div className="data-table-wrap">
            <table className="data-table">
              <thead>
                <tr>
                  <th style={{ width: '70px' }}>ID</th>
                  <th style={{ width: '170px' }}>Artist</th>
                  <th>Title</th>
                  <th style={{ width: '150px' }}>Genre</th>
                  <th style={{ width: '170px' }}>Album</th>
                  <th style={{ width: '60px' }}>Year</th>
                  <th style={{ width: '80px' }}>Duration</th>
                  <th className="actions-column wide-actions">Actions</th>
                </tr>
              </thead>
              <tbody>
                {rows.length === 0 ? (
                  <tr>
                    <td className="empty-cell" colSpan={8}>No tracks.</td>
                  </tr>
                ) : (
                  rows.map((row) => (
                    <tr key={row.id}>
                      <td>{row.id}</td>
                      <td>{row.artist || '—'}</td>
                      <td>{row.title || '—'}</td>
                      <td>{row.genre || '—'}</td>
                      <td>{row.album || '—'}</td>
                      <td>{row.year || '—'}</td>
                      <td>{formatDuration(row.duration)}</td>
                      <td className="row-actions">
                        <a className="ghost-button" href={`/tracks/cue/${row.id}`}>Cue</a>
                        <button
                          className="ghost-button"
                          type="button"
                          onClick={() => { setFormError(null); setEditTrack(row); }}
                        >
                          Edit
                        </button>
                        <button
                          className="danger-button"
                          type="button"
                          onClick={() => { setError(null); setDeleteTarget(row); }}
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <div className="pagination">
            <button
              className="ghost-button"
              disabled={page <= 1}
              type="button"
              onClick={() => setPage((p) => p - 1)}
            >
              ← Prev
            </button>
            <span className="pagination-info">
              Page {page} of {totalPages} &mdash; {total.toLocaleString()} total
            </span>
            <button
              className="ghost-button"
              disabled={page >= totalPages}
              type="button"
              onClick={() => setPage((p) => p + 1)}
            >
              Next →
            </button>
          </div>
        </>
      )}

      {editTrack ? (
        <TrackEditModal
          artists={artists}
          error={formError}
          genres={genres}
          saving={saving}
          subcategories={subcategories}
          track={editTrack}
          onClose={() => setEditTrack(null)}
          onSubmit={saveTrack}
        />
      ) : null}

      {importDraft ? (
        <TrackEditModal
          artists={artists}
          error={formError}
          genres={genres}
          mode="create"
          saving={saving}
          subcategories={subcategories}
          track={importDraft}
          onClose={() => setImportDraft(null)}
          onSubmit={saveImportedTrack}
        />
      ) : null}

      {scanFolderOpen ? (
        <ScanFolderModal
          genres={genres}
          stationId={stationId}
          subcategories={subcategories}
          onClose={() => setScanFolderOpen(false)}
          onFinished={loadTracks}
        />
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={deleting}
          message={`Delete "${deleteTarget.title}"? This action is irreversible.`}
          title="Delete Track"
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

function formatDuration(seconds) {
  if (seconds == null) return '—';
  const total = Math.round(seconds);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

async function fetchJson(url, options = {}) {
  const isFormData = options.body instanceof FormData;
  const response = await fetch(url, {
    ...options,
    headers: isFormData
      ? { ...(options.headers || {}) }
      : { 'Content-Type': 'application/json', ...(options.headers || {}) }
  });
  if (response.status === 204) return {};
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
  return payload;
}
