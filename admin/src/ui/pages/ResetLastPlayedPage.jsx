import { useState } from 'react';

const TARGETS = [
  { key: 'tracks',  label: 'Tracks' },
  { key: 'artists', label: 'Artists' }
];

export function ResetLastPlayedPage() {
  const [selected, setSelected] = useState({});
  const [confirming, setConfirming] = useState(false);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);

  const targets = TARGETS.filter((t) => selected[t.key]).map((t) => t.key);
  const canReset = targets.length > 0 && !running;

  function toggleTarget(key) {
    setSelected((prev) => ({ ...prev, [key]: !prev[key] }));
    setConfirming(false);
    setResult(null);
    setError(null);
  }

  function handleResetClick() {
    setConfirming(true);
  }

  function handleCancel() {
    setConfirming(false);
  }

  async function handleConfirm() {
    setRunning(true);
    setConfirming(false);
    setResult(null);
    setError(null);
    try {
      const response = await fetch('/api/reset-last-played', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ targets })
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || `Error ${response.status}`);
      setResult(payload.updated);
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
          <h2>Reset Last Played</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <p style={{ color: 'var(--color-text-muted, #888)', marginBottom: '1.5rem' }}>
          Clears the last played date for the selected items.
          The rotation engine will treat them as never played.
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', marginBottom: '2rem' }}>
          {TARGETS.map((t) => (
            <label key={t.key} style={{ display: 'flex', alignItems: 'center', gap: '0.6rem', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={!!selected[t.key]}
                onChange={() => toggleTarget(t.key)}
              />
              <span>{t.label}</span>
            </label>
          ))}
        </div>

        {error ? <div className="form-error" style={{ marginBottom: '1rem' }}>{error}</div> : null}

        {result ? (
          <div style={{ marginBottom: '1.5rem', color: '#16a34a', fontWeight: 600 }}>
            ✓ Reset complete —{' '}
            {Object.entries(result).map(([key, count]) => (
              <span key={key}>{TARGETS.find((t) => t.key === key)?.label}: <strong>{count}</strong> row{count !== 1 ? 's' : ''} </span>
            ))}
          </div>
        ) : null}

        {confirming ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', padding: '0.75rem 1rem', background: 'var(--color-surface, #f5f5f5)', borderRadius: '6px', marginBottom: '1rem' }}>
            <span style={{ color: '#b45309', fontWeight: 600 }}>
              Reset {targets.map((k) => TARGETS.find((t) => t.key === k)?.label).join(' & ')}? This cannot be undone.
            </span>
            <button className="danger-button" type="button" onClick={handleConfirm}>
              Yes, reset
            </button>
            <button className="secondary-button" type="button" onClick={handleCancel}>
              Cancel
            </button>
          </div>
        ) : (
          <button
            className="danger-button"
            disabled={!canReset}
            type="button"
            onClick={handleResetClick}
          >
            {running ? 'Resetting...' : 'Reset'}
          </button>
        )}
      </section>
    </section>
  );
}
