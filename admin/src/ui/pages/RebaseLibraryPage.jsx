import { useEffect, useState } from 'react';
import { useStation } from '../StationContext.jsx';

function stationSlug(name) {
  return String(name || '')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
}

export function RebaseLibraryPage() {
  const { stationId, stations } = useStation();
  const [oldPath, setOldPath] = useState('');
  const [newPath, setNewPath] = useState('');
  const [confirm, setConfirm] = useState(false);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);

  const currentStation = stations.find((s) => String(s.id) === String(stationId));
  const slug = stationSlug(currentStation?.name);
  const pathPlaceholder = slug ? `/mnt/nas01/OpenStudio/Library/${slug}` : '/mnt/nas01/OpenStudio/Library';

  useEffect(() => {
    if (currentStation?.library_path) {
      setOldPath(currentStation.library_path);
    }
  }, [currentStation?.library_path]);

  async function handleRebase() {
    setRunning(true);
    setResult(null);
    setError(null);
    setConfirm(false);
    try {
      const response = await fetch('/api/library/rebase', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ station_id: Number(stationId), old_path: oldPath, new_path: newPath })
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || `Request failed: ${response.status}`);
      setResult(payload);
    } catch (err) {
      setError(err.message);
    } finally {
      setRunning(false);
    }
  }

  const canSubmit = oldPath.trim() && newPath.trim() && oldPath.trim() !== newPath.trim() && stationId && !running;

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Admin</p>
          <h2>Rebase Library</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div
          style={{
            background: '#7f1d1d',
            border: '2px solid #dc2626',
            borderRadius: '6px',
            padding: '1rem 1.25rem',
            marginBottom: '1.5rem'
          }}
        >
          <strong style={{ color: '#fca5a5', fontSize: '1rem', display: 'block', marginBottom: '0.5rem' }}>
            ⚠ WARNING — FOR EXPERIENCED USERS ONLY
          </strong>
          <p style={{ color: '#fecaca', margin: '0 0 0.5rem' }}>
            This operation directly modifies the database. It replaces the path prefix of{' '}
            <strong>all tracks</strong> whose path starts with the old path, and updates the
            Library Path of the selected station.
          </p>
          <p style={{ color: '#fecaca', margin: '0 0 0.5rem' }}>
            Typical use case: moving the library to a different volume (NAS, RAID array, external drive).
          </p>
          <p style={{ color: '#f87171', fontWeight: 600, margin: 0 }}>
            This operation does NOT move files on disk. You must move the files manually to the
            new path BEFORE or AFTER running this operation.
          </p>
        </div>

        <form
          className="database-form"
          onSubmit={(e) => { e.preventDefault(); setConfirm(true); }}
        >
          <label>
            <span>Station</span>
            <input value={currentStation?.name || '(no station selected)'} readOnly disabled />
          </label>
          <label>
            <span>Old Path</span>
            <input
              value={oldPath}
              onChange={(e) => setOldPath(e.target.value)}
              placeholder={pathPlaceholder}
              spellCheck={false}
            />
          </label>
          <label>
            <span>New Path</span>
            <input
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              placeholder={pathPlaceholder}
              spellCheck={false}
            />
          </label>

          {error ? <div className="form-error">{error}</div> : null}

          {result ? (
            <div style={{ padding: '0.75rem 1rem', background: 'var(--color-surface, #111)', borderRadius: '4px', marginTop: '0.5rem' }}>
              <p style={{ color: '#16a34a', fontWeight: 600, margin: '0 0 0.25rem' }}>
                ✓ Rebase complete
              </p>
              <p style={{ color: 'var(--color-text-muted, #888)', margin: '0 0 0.25rem' }}>
                Tracks updated: <strong style={{ color: 'var(--color-text, #eee)' }}>{result.tracksUpdated}</strong>
              </p>
              <p style={{ color: 'var(--color-text-muted, #888)', margin: '0 0 0.75rem' }}>
                Station library path updated: <strong style={{ color: 'var(--color-text, #eee)' }}>{result.stationUpdated ? 'Yes' : 'No'}</strong>
              </p>
              <p style={{ color: '#fbbf24', fontWeight: 600, margin: 0 }}>
                Remember to physically move the files to the new path.
              </p>
            </div>
          ) : null}

          <div className="form-actions">
            <button
              className="danger-button"
              type="submit"
              disabled={!canSubmit}
            >
              {running ? 'Rebasing...' : 'Rebase Library'}
            </button>
          </div>
        </form>
      </section>

      {confirm ? (
        <div className="modal-overlay">
          <div className="modal-box">
            <h3>Confirm rebase</h3>
            <p>
              All tracks whose path starts with:<br />
              <code style={{ color: '#f87171', wordBreak: 'break-all' }}>{oldPath}</code>
            </p>
            <p>
              …will be updated to start with:<br />
              <code style={{ color: '#86efac', wordBreak: 'break-all' }}>{newPath}</code>
            </p>
            <p>
              The Library Path of station <strong>{currentStation?.name}</strong> will also be updated.
            </p>
            <p style={{ color: '#f87171', fontWeight: 600 }}>
              This action cannot be undone without a database backup. Are you sure?
            </p>
            <div className="modal-actions">
              <button className="danger-button" onClick={handleRebase}>Confirm rebase</button>
              <button className="ghost-button" onClick={() => setConfirm(false)}>Cancel</button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
