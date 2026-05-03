import { useEffect, useMemo, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';

const HOURS = Array.from({ length: 24 }, (_, hour) => hour);
const COVERAGE_DAYS = 42;
const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

export function PlaylistsPage() {
  const today = useMemo(() => dateInputValue(new Date()), []);
  const [fromDate, setFromDate] = useState(today);
  const [fromHour, setFromHour] = useState(0);
  const [toDate, setToDate] = useState(today);
  const [toHour, setToHour] = useState(23);
  const [error, setError] = useState(null);
  const [job, setJob] = useState(null);
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [coverage, setCoverage] = useState({ rows: [], timezone: '' });
  const [coverageError, setCoverageError] = useState(null);

  const running = job && !['completed', 'failed'].includes(job.status);
  const progress = job?.total ? Math.round((job.processed / job.total) * 100) : 0;
  const weeks = useMemo(() => buildCoverageWeeks(COVERAGE_DAYS, coverage.rows), [coverage.rows]);

  useEffect(() => {
    loadCoverage();
  }, []);

  useEffect(() => {
    if (!job || ['completed', 'failed'].includes(job.status)) return undefined;

    const timer = setInterval(async () => {
      try {
        const payload = await fetchJson(`/api/playlists/generate/${job.id}`);
        setJob(payload.job);
        if (payload.job.status === 'completed') loadCoverage();
      } catch (err) {
        setError(err.message);
      }
    }, 500);

    return () => clearInterval(timer);
  }, [job?.id, job?.status]);

  async function generate(event) {
    event.preventDefault();
    setError(null);
    setJob(null);

    try {
      const payload = await fetchJson('/api/playlists/generate', {
        method: 'POST',
        body: JSON.stringify({
          from_date: fromDate,
          from_hour: fromHour,
          to_date: toDate,
          to_hour: toHour
        })
      });
      setJob(payload.job);
    } catch (err) {
      setError(err.message);
    }
  }

  async function deleteQueue() {
    setDeleting(true);
    setError(null);

    try {
      const payload = await fetchJson('/api/playlists/queue', {
        method: 'DELETE',
        body: JSON.stringify({
          from_date: fromDate,
          from_hour: fromHour,
          to_date: toDate,
          to_hour: toHour
        })
      });
      setDeleteConfirm(false);
      setJob({
        id: `delete-${Date.now()}`,
        status: 'completed',
        total: 1,
        processed: 1,
        created: 0,
        skipped: 0,
        skippedHours: 0,
        current: '',
        messages: [{
          at: new Date().toISOString(),
          message: `Deleted ${payload.deleted || 0} queued entr${payload.deleted === 1 ? 'y' : 'ies'} from ${rangeLabel()}.`
        }]
      });
      await loadCoverage();
    } catch (err) {
      setError(err.message);
    } finally {
      setDeleting(false);
    }
  }

  async function loadCoverage() {
    setCoverageError(null);
    try {
      const payload = await fetchJson(`/api/playlists/coverage?days=${COVERAGE_DAYS}`);
      setCoverage({
        rows: payload.rows || [],
        timezone: payload.timezone || ''
      });
    } catch (err) {
      setCoverageError(err.message);
    }
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Automation</p>
          <h2>Playlists</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Generator</p>
            <h2>Generate Queue</h2>
          </div>
        </div>

        <form className="playlist-generator" onSubmit={generate}>
          <div className="form-row">
            <label>
                <span>From date</span>
              <input
                min={today}
                required
                type="date"
                value={fromDate}
                onChange={(event) => {
                  setFromDate(event.target.value);
                  if (event.target.value > toDate) setToDate(event.target.value);
                }}
              />
            </label>
            <label>
              <span>From hour</span>
              <select value={fromHour} onChange={(event) => setFromHour(Number(event.target.value))}>
                {HOURS.map((hour) => (
                  <option key={hour} value={hour}>{formatHour(hour)}</option>
                ))}
              </select>
            </label>
          </div>

          <div className="form-row">
            <label>
                <span>To date</span>
              <input
                min={fromDate || today}
                required
                type="date"
                value={toDate}
                onChange={(event) => setToDate(event.target.value)}
              />
            </label>
            <label>
              <span>To hour</span>
              <select value={toHour} onChange={(event) => setToHour(Number(event.target.value))}>
                {HOURS.map((hour) => (
                  <option key={hour} value={hour}>{formatHour(hour)}</option>
                ))}
              </select>
            </label>
          </div>

          {error ? <div className="form-error">{error}</div> : null}

          <div className="form-actions">
            <button className="primary-button" disabled={running} type="submit">
              {running ? 'Generating...' : 'Generate'}
            </button>
            <button
              className="danger-button solid scary-button"
              disabled={running || deleting}
              onClick={() => {
                setError(null);
                setDeleteConfirm(true);
              }}
              type="button"
            >
              Delete
            </button>
          </div>
        </form>

        {job ? (
          <section className="generation-progress">
            <div className="progress-header">
              <strong>{job.status === 'completed' ? 'Completed' : job.status === 'failed' ? 'Failed' : 'Generating...'}</strong>
              <span>{job.processed} / {job.total || 0} hours</span>
            </div>
            <div className="progress-bar">
              <div style={{ width: `${progress}%` }} />
            </div>

            <div className="scan-counters">
              <span>Created: <strong>{job.created}</strong></span>
              <span>Skipped slots: <strong>{job.skipped}</strong></span>
              <span>Skipped hours: <strong>{job.skippedHours}</strong></span>
            </div>

            {job.current ? (
              <div className="import-summary">
                <span>{job.current}</span>
              </div>
            ) : null}

            <div className="job-messages">
              {job.messages.map((entry, index) => (
                <div key={`${entry.at}-${index}`}>{entry.message}</div>
              ))}
            </div>
          </section>
        ) : null}
      </section>

      <section className="playlist-panel playlist-coverage">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Coverage</p>
            <h2>Generated Hours</h2>
          </div>
          <span>{coverage.timezone || 'Timezone loading...'}</span>
        </div>

        {coverageError ? <div className="table-error">{coverageError}</div> : null}

        <div className="coverage-weeks">
          {weeks.map((week) => (
            <section className="coverage-week" key={week.label}>
              <header>{week.label}</header>
              <div className="coverage-days">
                {week.days.map((day) => (
                  <div className="coverage-day" key={day.date}>
                    <div className="coverage-day-header">
                      <strong>{DAY_NAMES[day.weekday]}</strong>
                      <span>{formatShortDate(day.date)}</span>
                    </div>
                    <div className="coverage-hours">
                      {HOURS.map((hour) => {
                        const generated = day.generatedHours.has(hour);
                        return (
                          <span
                            className={generated ? 'coverage-hour generated' : 'coverage-hour'}
                            key={hour}
                            title={`${day.date} ${formatHour(hour)}${generated ? ' generated' : ' empty'}`}
                          />
                        );
                      })}
                    </div>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>
      </section>

      {deleteConfirm ? (
        <ConfirmDialog
          busy={deleting}
          message={`WARNING !!!!! This will permanently delete all queued playlist entries from ${rangeLabel()}. This action cannot be undone.`}
          title="Delete Generated Playlists"
          onCancel={() => setDeleteConfirm(false)}
          onConfirm={deleteQueue}
        />
      ) : null}
    </section>
  );

  function rangeLabel() {
    return `${fromDate} ${formatHour(fromHour)} to ${toDate} ${formatHour(toHour)}`;
  }
}

function dateInputValue(date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function addDays(dateValue, days) {
  const [year, month, day] = dateValue.split('-').map(Number);
  const date = new Date(year, month - 1, day + days);
  return dateInputValue(date);
}

function buildCoverageWeeks(days, rows) {
  const generated = new Map();
  for (const row of rows) {
    const key = String(row.date);
    if (!generated.has(key)) generated.set(key, new Set());
    generated.get(key).add(Number(row.hour));
  }

  const today = dateInputValue(new Date());
  const dates = Array.from({ length: days }, (_, index) => {
    const date = addDays(today, index);
    return {
      date,
      weekday: weekdayForDate(date),
      generatedHours: generated.get(date) || new Set()
    };
  });

  const weeks = [];
  for (let i = 0; i < dates.length; i += 7) {
    const weekDays = dates.slice(i, i + 7);
    weeks.push({
      label: `${formatShortDate(weekDays[0].date)} - ${formatShortDate(weekDays[weekDays.length - 1].date)}`,
      days: weekDays
    });
  }
  return weeks;
}

function weekdayForDate(value) {
  const [year, month, day] = value.split('-').map(Number);
  return new Date(year, month - 1, day).getDay();
}

function formatShortDate(value) {
  const [, month, day] = value.split('-');
  return `${day}/${month}`;
}

function pad(value) {
  return String(value).padStart(2, '0');
}

function formatHour(hour) {
  return `${pad(hour)}:00`;
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
