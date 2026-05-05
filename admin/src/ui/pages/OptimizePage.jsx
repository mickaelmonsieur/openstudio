import { useState } from 'react';

export function OptimizePage() {
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState(null);

  async function handleOptimize() {
    setRunning(true);
    setDone(false);
    setError(null);
    try {
      const response = await fetch('/api/optimize', { method: 'POST' });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || `Error ${response.status}`);
      setDone(true);
    } catch (err) {
      setError(err.message);
    } finally {
      setRunning(false);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Utilities</p>
          <h2>Optimize Database</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <p style={{ color: 'var(--color-text-muted, #888)', marginBottom: '0.5rem' }}>
          Runs a full maintenance cycle on the PostgreSQL database:
        </p>
        <ul style={{ color: 'var(--color-text-muted, #888)', marginBottom: '1.5rem', paddingLeft: '1.25rem', lineHeight: '1.8' }}>
          <li><strong>VACUUM FULL</strong> — reclaims disk space from deleted rows</li>
          <li><strong>ANALYZE</strong> — updates query planner statistics</li>
          <li><strong>REINDEX</strong> — rebuilds all indexes</li>
        </ul>
        <p style={{ color: 'var(--color-text-muted, #888)', marginBottom: '2rem', fontSize: '0.875em' }}>
          Tables are locked during the operation. Run during low-activity periods.
        </p>

        {error ? <div className="form-error" style={{ marginBottom: '1rem' }}>{error}</div> : null}

        {done ? (
          <div style={{ marginBottom: '1.5rem', color: '#16a34a', fontWeight: 600 }}>
            ✓ Database optimized successfully.
          </div>
        ) : null}

        <button
          className="primary-button"
          disabled={running}
          type="button"
          onClick={handleOptimize}
        >
          {running ? 'Optimizing...' : 'Optimize Database'}
        </button>
      </section>
    </section>
  );
}
