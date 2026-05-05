import { useState } from 'react';

const TARGETS = [
  { key: 'queue_played', label: 'Queue' },
  { key: 'play_log',     label: 'Play Log' },
  { key: 'automix_log',  label: 'Auto Mix Log' }
];

const DAYS_OPTIONS = [30, 90, 180, 365];

export function CleanupPage() {
  const [selected, setSelected] = useState({});
  const [days, setDays] = useState(90);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);

  function toggleTarget(key) {
    setSelected((prev) => ({ ...prev, [key]: !prev[key] }));
    setResult(null);
    setError(null);
  }

  const targets = TARGETS.filter((t) => selected[t.key]).map((t) => t.key);
  const canDelete = targets.length > 0 && !running;

  async function handleDelete() {
    if (!canDelete) return;
    setRunning(true);
    setResult(null);
    setError(null);
    try {
      const response = await fetch('/api/purge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ targets, days: Number(days) })
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || `Error ${response.status}`);
      setResult(payload.deleted);
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
          <h2>Cleanup</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div className="playlist-generator">
          <p className="panel-kicker">Records to delete</p>

          {TARGETS.map((t) => (
            <label key={t.key} style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: '8px' }}>
              <input
                type="checkbox"
                checked={!!selected[t.key]}
                onChange={() => toggleTarget(t.key)}
                style={{ width: '16px', height: '16px', accentColor: '#6857d8', flexShrink: 0 }}
              />
              <span>{t.label}</span>
            </label>
          ))}

          <label>
            <span>Older than</span>
            <select
              value={days}
              onChange={(e) => { setDays(e.target.value); setResult(null); setError(null); }}
            >
              {DAYS_OPTIONS.map((d) => (
                <option key={d} value={d}>{d} days</option>
              ))}
            </select>
          </label>

          {error ? <div className="form-error">{error}</div> : null}

          {result ? (
            <div className="scan-counters">
              {Object.entries(result).map(([key, count]) => (
                <span key={key}>{TARGETS.find((t) => t.key === key)?.label ?? key}: <strong>{count}</strong></span>
              ))}
            </div>
          ) : null}

          <div className="form-actions">
            <button
              className="danger-button"
              disabled={!canDelete}
              type="button"
              onClick={handleDelete}
            >
              {running ? 'Deleting...' : 'Delete forever'}
            </button>
          </div>
        </div>
      </section>
    </section>
  );
}
