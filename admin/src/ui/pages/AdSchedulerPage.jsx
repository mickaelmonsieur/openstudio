import { useContext, useEffect, useMemo, useState } from 'react';
import { StationContext } from '../StationContext.jsx';

const HOURS = Array.from({ length: 24 }, (_, hour) => hour);
const COVERAGE_DAYS = 7;
const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

export function AdSchedulerPage() {
  const { stationId } = useContext(StationContext);
  const today = useMemo(() => dateInputValue(new Date()), []);
  const [fromDate, setFromDate] = useState(today);
  const [fromHour, setFromHour] = useState(0);
  const [toDate, setToDate] = useState(today);
  const [toHour, setToHour] = useState(23);
  const [error, setError] = useState(null);
  const [summary, setSummary] = useState(null);
  const [generating, setGenerating] = useState(false);
  const [coverageStartDate, setCoverageStartDate] = useState(today);
  const [coverage, setCoverage] = useState({ rows: [], timezone: '' });
  const [coverageError, setCoverageError] = useState(null);

  const week = useMemo(() => buildCoverageWeek(COVERAGE_DAYS, coverageStartDate, coverage.rows), [coverageStartDate, coverage.rows]);
  const coverageEndDate = useMemo(() => addDays(coverageStartDate, COVERAGE_DAYS - 1), [coverageStartDate]);
  const totals = useMemo(() => coverage.rows.reduce((acc, row) => {
    acc.total += 1;
    acc[row.status] = (acc[row.status] || 0) + 1;
    return acc;
  }, { total: 0, filled: 0, partial: 0, empty: 0 }), [coverage.rows]);

  useEffect(() => {
    loadCoverage();
  }, [coverageStartDate]);

  async function generate(event) {
    event.preventDefault();
    setGenerating(true);
    setError(null);
    setSummary(null);

    try {
      const payload = await fetchJson('/api/ad-scheduler/generate', {
        method: 'POST',
        body: JSON.stringify({
          from_date: fromDate,
          from_hour: fromHour,
          to_date: toDate,
          to_hour: toHour,
          station_id: stationId || null
        })
      });
      setSummary(payload.summary);
      await loadCoverage();
    } catch (err) {
      setError(err.message);
    } finally {
      setGenerating(false);
    }
  }

  async function loadCoverage() {
    setCoverageError(null);
    try {
      const params = new URLSearchParams({
        days: String(COVERAGE_DAYS),
        start_date: coverageStartDate
      });
      const payload = await fetchJson(`/api/ad-scheduler/coverage?${params.toString()}`);
      setCoverage({
        rows: payload.rows || [],
        timezone: payload.timezone || ''
      });
    } catch (err) {
      setCoverageError(err.message);
    }
  }

  function moveCoverage(days) {
    setCoverageStartDate((current) => addDays(current, days));
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Automation</p>
          <h2>Ad Scheduler</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Generator</p>
            <h2>Fill Ad Breaks</h2>
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
            <button className="primary-button" disabled={generating} type="submit">
              {generating ? 'Generating...' : 'Generate'}
            </button>
          </div>
        </form>

        {summary ? (
          <section className="generation-progress">
            <div className="scan-counters">
              <span>Screens: <strong>{summary.screens}</strong></span>
              <span>Filled: <strong>{summary.filled}</strong></span>
              <span>Partial: <strong>{summary.partial}</strong></span>
              <span>Spots: <strong>{summary.inserted}</strong></span>
            </div>
            <div className="job-messages">
              {(summary.messages || []).slice().reverse().map((message, index) => (
                <div key={`${message}-${index}`} className="msg-info">{message}</div>
              ))}
            </div>
          </section>
        ) : null}
      </section>

      <section className="playlist-panel playlist-coverage">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Coverage</p>
            <h2>Ad Breaks This Week</h2>
          </div>
          <div className="coverage-nav">
            <button className="ghost-button table-icon-button" type="button" title="Previous week" aria-label="Previous week" onClick={() => moveCoverage(-COVERAGE_DAYS)}>
              <i className="bi bi-arrow-left" aria-hidden="true" />
            </button>
            <span>{formatShortDate(coverageStartDate)} - {formatShortDate(coverageEndDate)} · {coverage.timezone || 'Timezone loading...'}</span>
            <button className="ghost-button table-icon-button" type="button" title="Next week" aria-label="Next week" onClick={() => moveCoverage(COVERAGE_DAYS)}>
              <i className="bi bi-arrow-right" aria-hidden="true" />
            </button>
          </div>
        </div>

        <div className="ad-coverage-legend">
          <span><i className="ad-dot filled" /> Filled {totals.filled}</span>
          <span><i className="ad-dot partial" /> With ads {totals.partial}</span>
          <span><i className="ad-dot empty" /> Empty {totals.empty}</span>
        </div>

        {coverageError ? <div className="table-error">{coverageError}</div> : null}

        <div className="ad-coverage-grid">
          {week.map((day) => (
            <section className="ad-coverage-day" key={day.date}>
              <header>
                <strong>{DAY_NAMES[day.weekday]}</strong>
                <span>{formatShortDate(day.date)}</span>
              </header>
              <div className="ad-coverage-hours">
                {HOURS.map((hour) => {
                  const screens = day.hours.get(hour) || [];
                  return (
                    <div className="ad-coverage-hour" key={hour}>
                      <span>{formatHour(hour)}</span>
                      <div className="ad-break-stack">
                        {screens.length > 0 ? screens.map((screen) => (
                          <i
                            className={`ad-break-pill ${screen.status}`}
                            key={screen.id}
                            title={`${day.date} ${screen.start_time} - ${screen.ad_count} ad(s), ${screen.filler_count} filler(s)`}
                          />
                        )) : <i className="ad-break-placeholder" />}
                      </div>
                    </div>
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      </section>
    </section>
  );
}

function dateInputValue(date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function addDays(dateValue, days) {
  const [year, month, day] = dateValue.split('-').map(Number);
  const date = new Date(year, month - 1, day + days);
  return dateInputValue(date);
}

function buildCoverageWeek(days, startDate, rows) {
  const byHour = new Map();
  for (const row of rows) {
    const key = `${row.date}-${row.hour}`;
    if (!byHour.has(key)) byHour.set(key, []);
    byHour.get(key).push(row);
  }

  return Array.from({ length: days }, (_, index) => {
    const date = addDays(startDate, index);
    const hours = new Map();
    for (const hour of HOURS) {
      hours.set(hour, byHour.get(`${date}-${hour}`) || []);
    }
    return { date, weekday: weekdayForDate(date), hours };
  });
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
  if (!response.ok) {
    throw new Error(payload.error || `Request failed with ${response.status}`);
  }
  return payload;
}
