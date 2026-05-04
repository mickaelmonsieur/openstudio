import { useEffect, useState } from 'react';

export function FileScanPage() {
  const [job, setJob] = useState(null);
  const [error, setError] = useState(null);

  const running = job && !['completed', 'failed'].includes(job.status);
  const progress = job?.total > 0 ? Math.round((job.processed / job.total) * 100) : 0;

  useEffect(() => {
    if (!running) return undefined;

    const timer = setInterval(async () => {
      try {
        const payload = await fetchJson(`/api/file-scan/${job.id}`);
        setJob(payload.job);
      } catch (err) {
        setError(err.message);
      }
    }, 400);

    return () => clearInterval(timer);
  }, [job?.id, running]);

  async function startScan() {
    setError(null);
    setJob(null);
    try {
      const payload = await fetchJson('/api/file-scan', { method: 'POST' });
      setJob(payload.job);
    } catch (err) {
      setError(err.message);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Admin</p>
          <h2>Scan Files</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Verification</p>
            <h2>Check Track Files</h2>
          </div>
        </div>

        <p style={{ color: 'var(--color-text-muted, #888)', marginBottom: '1rem' }}>
          Scans every track in the database and checks whether its file exists on disk.
        </p>

        {error ? <div className="form-error">{error}</div> : null}

        <div className="form-actions">
          <button
            className="primary-button"
            disabled={running}
            type="button"
            onClick={startScan}
          >
            {running ? 'Scanning...' : 'Start Scan'}
          </button>
        </div>

        {job ? (
          <section className="generation-progress">
            <div className="progress-header">
              <strong>
                {job.status === 'completed' ? 'Completed' : job.status === 'failed' ? 'Failed' : 'Scanning...'}
              </strong>
              <span>{job.processed} / {job.total} tracks</span>
            </div>
            <div className="progress-bar">
              <div style={{ width: `${progress}%` }} />
            </div>

            <div className="scan-counters">
              <span style={{ color: '#16a34a' }}>OK: <strong>{job.ok}</strong></span>
              <span style={{ color: job.missing.length > 0 ? '#dc2626' : 'inherit' }}>
                Missing: <strong>{job.missing.length}{job.missing.length === 500 ? '+' : ''}</strong>
              </span>
              <span>Total: <strong>{job.total}</strong></span>
            </div>

            {job.error ? <div className="form-error">{job.error}</div> : null}

            {job.missing.length > 0 ? (
              <div className="job-messages" style={{ marginTop: '1rem' }}>
                <strong style={{ color: '#dc2626', display: 'block', marginBottom: '0.5rem' }}>
                  Missing files:
                </strong>
                {job.missing.map((entry) => (
                  <div key={entry.id} style={{ display: 'flex', gap: '0.75rem', padding: '0.25rem 0', borderBottom: '1px solid var(--color-border, #eee)' }}>
                    <span style={{ color: '#888', minWidth: '60px', flexShrink: 0 }}>#{entry.id}</span>
                    <span style={{ flexShrink: 0, maxWidth: '220px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={entry.label}>{entry.label}</span>
                    <span style={{ color: '#dc2626', fontFamily: 'monospace', fontSize: '0.85em', wordBreak: 'break-all' }}>{entry.path}</span>
                  </div>
                ))}
              </div>
            ) : null}

            {job.status === 'completed' && job.missing.length === 0 ? (
              <div style={{ marginTop: '1rem', color: '#16a34a', fontWeight: 600 }}>
                ✓ All {job.ok} files found on disk.
              </div>
            ) : null}
          </section>
        ) : null}
      </section>
    </section>
  );
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
