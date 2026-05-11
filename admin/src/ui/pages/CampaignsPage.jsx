import { useEffect, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const DAYS = [
  { id: 1, label: 'Monday' },
  { id: 2, label: 'Tuesday' },
  { id: 3, label: 'Wednesday' },
  { id: 4, label: 'Thursday' },
  { id: 5, label: 'Friday' },
  { id: 6, label: 'Saturday' },
  { id: 7, label: 'Sunday' }
];
const HOURS = Array.from({ length: 24 }, (_, index) => index);
const COLS = [
  { key: 'id',              label: 'ID',         width: '60px' },
  { key: 'advertiser_name', label: 'Advertiser', width: '160px' },
  { key: 'name',            label: 'Name' },
  { key: 'station_name',    label: 'Station',    width: '120px' },
  { key: 'tracks_count',    label: 'Tracks',     width: '75px' },
  { key: 'active',          label: 'Active',     width: '65px' },
  { key: 'start_date',      label: 'Start',      width: '100px' },
  { key: 'end_date',        label: 'End',        width: '100px' },
  { key: 'broadcast_count', label: 'Aired',      width: '65px' },
  { key: 'total_broadcasts',label: 'Total',      width: '65px' },
  { key: 'max_broadcasts_per_day', label: 'Daily Max', width: '85px' },
  { key: 'min_broadcast_gap_minutes', label: 'Gap Min.', width: '85px' },
  { key: 'splitting_enabled', label: 'Split', width: '65px' },
  { key: 'split_min_spots_between', label: 'Split Gap', width: '85px' }
];

const LIMIT = 50;

function emptyForm() {
  const year = new Date().getFullYear();
  return {
    advertiser_id: '',
    name: '',
    station_id: '',
    total_broadcasts: 0,
    max_broadcasts_per_day: 0,
    min_broadcast_gap_minutes: 0,
    splitting_enabled: false,
    split_min_spots_between: 1,
    active: true,
    start_date: `${year}-01-01`,
    end_date: `${year}-12-31`
  };
}

export function CampaignsPage() {
  const [rows, setRows]               = useState([]);
  const [total, setTotal]             = useState(0);
  const [page, setPage]               = useState(1);
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterAdvertiser, setFilterAdvertiser] = useState('');
  const [filterActive, setFilterActive] = useState('');
  const [advertisers, setAdvertisers] = useState([]);
  const [stations, setStations]       = useState([]);
  const [loading, setLoading]         = useState(true);
  const [error, setError]             = useState(null);
  const [modal, setModal]             = useState(null);
  const [form, setForm]               = useState(emptyForm);
  const [formError, setFormError]     = useState(null);
  const [saving, setSaving]           = useState(false);
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [hoursModal, setHoursModal]   = useState(null);
  const [hoursSet, setHoursSet]       = useState(new Set());
  const [hoursLoading, setHoursLoading] = useState(false);
  const [hoursSaving, setHoursSaving] = useState(false);
  const [hoursError, setHoursError]   = useState(null);
  const [hoursTab, setHoursTab]       = useState('weekly');
  const [calendarDates, setCalendarDates] = useState([]);
  const [calendarDateInput, setCalendarDateInput] = useState(localDateToday());

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

  useEffect(() => { load(); }, [page, searchQuery, filterAdvertiser, filterActive]);

  async function load() {
    setLoading(true); setError(null);
    try {
      const params = new URLSearchParams({ page: String(page), limit: String(LIMIT) });
      if (searchQuery) params.set('q', searchQuery);
      if (filterAdvertiser) params.set('advertiser_id', filterAdvertiser);
      if (filterActive) params.set('active', filterActive);
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
      max_broadcasts_per_day: row.max_broadcasts_per_day ?? 0,
      min_broadcast_gap_minutes: row.min_broadcast_gap_minutes ?? 0,
      splitting_enabled: Boolean(row.splitting_enabled),
      split_min_spots_between: row.split_min_spots_between ?? 1,
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

  async function openBroadcastHours(row) {
    setHoursModal(row);
    setHoursSet(new Set());
    setCalendarDates([]);
    setCalendarDateInput(localDateToday());
    setHoursTab('weekly');
    setHoursError(null);
    setHoursLoading(true);
    try {
      const [weeklyPayload, calendarPayload] = await Promise.all([
        fetchJson(`/api/campaigns/${row.id}/broadcast-hours`),
        fetchJson(`/api/campaigns/${row.id}/calendar-hours`)
      ]);
      setHoursSet(new Set((weeklyPayload.hours || []).map((slot) => hourKey(slot.iso_weekday, slot.hour))));
      setCalendarDates(groupCalendarHours(calendarPayload.hours || []));
    } catch (err) {
      setHoursError(err.message);
    } finally {
      setHoursLoading(false);
    }
  }

  function setAllHours(enabled) {
    setHoursSet(enabled ? new Set(allHourKeys()) : new Set());
  }

  function setBusinessHours() {
    const next = new Set();
    for (let day = 1; day <= 5; day += 1) {
      for (let hour = 8; hour <= 18; hour += 1) {
        next.add(hourKey(day, hour));
      }
    }
    setHoursSet(next);
  }

  function toggleHour(day, hour) {
    const key = hourKey(day, hour);
    setHoursSet((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  function setDay(day, enabled) {
    setHoursSet((prev) => {
      const next = new Set(prev);
      for (const hour of HOURS) {
        const key = hourKey(day, hour);
        if (enabled) next.add(key);
        else next.delete(key);
      }
      return next;
    });
  }

  function setColumnHour(hour, enabled) {
    setHoursSet((prev) => {
      const next = new Set(prev);
      for (const day of DAYS) {
        const key = hourKey(day.id, hour);
        if (enabled) next.add(key);
        else next.delete(key);
      }
      return next;
    });
  }

  function addCalendarDate() {
    const broadcastDate = calendarDateInput.trim();
    if (!/^\d{4}-\d{2}-\d{2}$/.test(broadcastDate)) {
      setHoursError('Calendar date is invalid.');
      return;
    }
    if (calendarDates.some((row) => row.broadcast_date === broadcastDate)) {
      setHoursError('Calendar date already exists.');
      return;
    }

    setHoursError(null);
    setCalendarDates((prev) => [...prev, { broadcast_date: broadcastDate, hours: new Set() }]
      .sort((a, b) => a.broadcast_date.localeCompare(b.broadcast_date)));
  }

  function removeCalendarDate(broadcastDate) {
    setCalendarDates((prev) => prev.filter((row) => row.broadcast_date !== broadcastDate));
  }

  function toggleCalendarHour(broadcastDate, hour) {
    setCalendarDates((prev) => prev.map((row) => {
      if (row.broadcast_date !== broadcastDate) return row;
      const nextHours = new Set(row.hours);
      if (nextHours.has(hour)) nextHours.delete(hour);
      else nextHours.add(hour);
      return { ...row, hours: nextHours };
    }));
  }

  function setCalendarDateHours(broadcastDate, enabled) {
    setCalendarDates((prev) => prev.map((row) => (
      row.broadcast_date === broadcastDate
        ? { ...row, hours: enabled ? new Set(HOURS) : new Set() }
        : row
    )));
  }

  function setCalendarDateBusinessHours(broadcastDate) {
    const businessHours = new Set();
    for (let hour = 8; hour <= 18; hour += 1) {
      businessHours.add(hour);
    }

    setCalendarDates((prev) => prev.map((row) => (
      row.broadcast_date === broadcastDate
        ? { ...row, hours: businessHours }
        : row
    )));
  }

  async function saveBroadcastHours() {
    if (!hoursModal) return;
    setHoursSaving(true);
    setHoursError(null);
    try {
      const hours = [...hoursSet].map(parseHourKey);
      const dates = calendarDatesToPayload(calendarDates);
      const [weeklyPayload, calendarPayload] = await Promise.all([
        fetchJson(`/api/campaigns/${hoursModal.id}/broadcast-hours`, {
          method: 'PUT',
          body: JSON.stringify({ hours })
        }),
        fetchJson(`/api/campaigns/${hoursModal.id}/calendar-hours`, {
          method: 'PUT',
          body: JSON.stringify({ dates })
        })
      ]);
      setHoursSet(new Set((weeklyPayload.hours || []).map((slot) => hourKey(slot.iso_weekday, slot.hour))));
      setCalendarDates(groupCalendarHours(calendarPayload.hours || []));
      setHoursModal(null);
    } catch (err) {
      setHoursError(err.message);
    } finally {
      setHoursSaving(false);
    }
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

      <div className="track-filters">
        <label>
          Advertiser
          <select value={filterAdvertiser} onChange={(e) => { setFilterAdvertiser(e.target.value); setPage(1); }}>
            <option value="">All</option>
            {advertisers.map((advertiser) => (
              <option key={advertiser.id} value={advertiser.id}>{advertiser.name}</option>
            ))}
          </select>
        </label>
        <label>
          Active
          <select value={filterActive} onChange={(e) => { setFilterActive(e.target.value); setPage(1); }}>
            <option value="">All</option>
            <option value="1">Active</option>
            <option value="0">Inactive</option>
          </select>
        </label>
      </div>

      {loading ? <div className="table-loading">Loading...</div> : (
        <DataTable columns={COLS} primaryKey="id" rows={rows}
          renderRowActions={(row) => (
            <button
              aria-label="Broadcast Hours"
              className="ghost-button table-icon-button"
              onClick={(event) => { event.stopPropagation(); openBroadcastHours(row); }}
              title="Broadcast Hours"
              type="button"
            >
              <i className="bi bi-calendar-week" aria-hidden="true" />
            </button>
          )}
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
          <section className="modal-panel campaign-modal" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">{modal === 'add' ? 'Add' : 'Edit'}</p>
                <h2>Campaign</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
            </header>
            <form className="resource-form campaign-form" onSubmit={save}>
              <label><span>Advertiser *</span>
                <select required value={form.advertiser_id} onChange={(e) => upd('advertiser_id', e.target.value)}>
                  <option value="">— select —</option>
                  {advertisers.map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
                </select>
              </label>
              <label><span>Name *</span>
                <input autoFocus maxLength={255} required value={form.name} onChange={(e) => upd('name', e.target.value)} />
              </label>
              <label><span>Station</span>
                <select value={form.station_id} onChange={(e) => upd('station_id', e.target.value)}>
                  <option value="">— any —</option>
                  {stations.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
              </label>
              <div className="form-row">
                <label><span>Start Date *</span>
                  <input required type="date" value={form.start_date} onChange={(e) => upd('start_date', e.target.value)} />
                </label>
                <label><span>End Date *</span>
                  <input required min={form.start_date || undefined} type="date" value={form.end_date} onChange={(e) => upd('end_date', e.target.value)} />
                </label>
              </div>
              <label><span>Total Broadcasts</span>
                <input type="number" min="0" value={form.total_broadcasts}
                  onChange={(e) => upd('total_broadcasts', Number(e.target.value))} />
              </label>
              <div className="form-row">
                <label><span>Max Broadcasts / Day</span>
                  <input type="number" min="0" value={form.max_broadcasts_per_day}
                    onChange={(e) => upd('max_broadcasts_per_day', Number(e.target.value))} />
                </label>
                <label><span>Minimum Gap (minutes)</span>
                  <input type="number" min="0" value={form.min_broadcast_gap_minutes}
                    onChange={(e) => upd('min_broadcast_gap_minutes', Number(e.target.value))} />
                </label>
              </div>
              <label className="checkbox-field"><span>Splitting</span>
                <input type="checkbox" checked={form.splitting_enabled} onChange={(e) => upd('splitting_enabled', e.target.checked)} />
              </label>
              {form.splitting_enabled ? (
                <label><span>Minimum Spots Between Split</span>
                  <input type="number" min="1" value={form.split_min_spots_between}
                    onChange={(e) => upd('split_min_spots_between', Number(e.target.value))} />
                </label>
              ) : null}
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

      {hoursModal ? (
        <div className="modal-backdrop">
          <section className="modal-panel campaign-hours-modal" role="dialog" aria-modal="true">
            <header className="modal-header">
              <div>
                <p className="panel-kicker">Broadcast Hours</p>
                <h2>{hoursModal.name}</h2>
              </div>
              <button className="icon-button" type="button" onClick={() => setHoursModal(null)}>×</button>
            </header>

            <div className="campaign-hours-tabs">
              <button className={hoursTab === 'weekly' ? 'active' : ''} type="button" onClick={() => setHoursTab('weekly')}>
                Weekly Rules
              </button>
              <button className={hoursTab === 'calendar' ? 'active' : ''} type="button" onClick={() => setHoursTab('calendar')}>
                Calendar Rules
              </button>
            </div>

            {hoursError ? <div className="form-error">{hoursError}</div> : null}
            {hoursLoading ? <div className="table-loading">Loading...</div> : (
              hoursTab === 'weekly' ? (
                <>
                  <div className="campaign-hours-actions">
                    <button className="ghost-button" type="button" onClick={() => setAllHours(true)}>All week</button>
                    <button className="ghost-button" type="button" onClick={() => setBusinessHours()}>Business hours</button>
                    <button className="ghost-button" type="button" onClick={() => setAllHours(false)}>Clear</button>
                    <span>{hoursSet.size} / 168 enabled</span>
                  </div>

                  <div className="campaign-hours-grid-wrap">
                    <div className="campaign-hours-grid">
                      <div className="campaign-hours-corner">Day / Hour</div>
                      {HOURS.map((hour) => {
                        const enabled = DAYS.every((day) => hoursSet.has(hourKey(day.id, hour)));
                        return (
                          <button
                            className={`campaign-hour-header ${enabled ? 'enabled' : ''}`}
                            key={hour}
                            title={`Toggle ${hour}:00 for all days`}
                            type="button"
                            onClick={() => setColumnHour(hour, !enabled)}
                          >
                            {hour}
                          </button>
                        );
                      })}

                      {DAYS.map((day) => {
                        const dayEnabled = HOURS.every((hour) => hoursSet.has(hourKey(day.id, hour)));
                        return (
                          <div className="campaign-hours-row" key={day.id}>
                            <button
                              className={`campaign-day-header ${dayEnabled ? 'enabled' : ''}`}
                              type="button"
                              onClick={() => setDay(day.id, !dayEnabled)}
                            >
                              {day.label}
                            </button>
                            {HOURS.map((hour) => {
                              const key = hourKey(day.id, hour);
                              const enabled = hoursSet.has(key);
                              return (
                                <button
                                  aria-label={`${day.label} ${hour}:00 ${enabled ? 'enabled' : 'disabled'}`}
                                  className={`campaign-hour-cell ${enabled ? 'enabled' : ''}`}
                                  key={key}
                                  type="button"
                                  onClick={() => toggleHour(day.id, hour)}
                                >
                                  {enabled ? '✓' : '×'}
                                </button>
                              );
                            })}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </>
              ) : (
                <div className="campaign-calendar-panel">
                  <div className="campaign-calendar-toolbar">
                    <label>
                      <span>Date</span>
                      <input type="date" value={calendarDateInput} onChange={(e) => setCalendarDateInput(e.target.value)} />
                    </label>
                    <button className="ghost-button" type="button" onClick={addCalendarDate}>Add Date</button>
                    <span>{calendarHoursCount(calendarDates)} selected hour(s)</span>
                  </div>

                  {calendarDates.length === 0 ? (
                    <div className="table-loading">No calendar rules.</div>
                  ) : (
                    <div className="campaign-calendar-list">
                      {calendarDates.map((row) => (
                        <section className="campaign-calendar-date" key={row.broadcast_date}>
                          <header>
                            <strong>{row.broadcast_date}</strong>
                            <span>{row.hours.size} / 24 enabled</span>
                            <button className="ghost-button" type="button" onClick={() => setCalendarDateHours(row.broadcast_date, true)}>All day</button>
                            <button className="ghost-button" type="button" onClick={() => setCalendarDateBusinessHours(row.broadcast_date)}>Business hours</button>
                            <button className="ghost-button" type="button" onClick={() => setCalendarDateHours(row.broadcast_date, false)}>Clear</button>
                            <button className="danger-button table-icon-button" title="Delete date" type="button" onClick={() => removeCalendarDate(row.broadcast_date)}>
                              <i className="bi bi-trash" aria-hidden="true" />
                            </button>
                          </header>
                          <div className="campaign-calendar-hours">
                            {HOURS.map((hour) => {
                              const enabled = row.hours.has(hour);
                              return (
                                <button
                                  aria-label={`${row.broadcast_date} ${hour}:00 ${enabled ? 'enabled' : 'disabled'}`}
                                  className={`campaign-hour-cell ${enabled ? 'enabled' : ''}`}
                                  key={hour}
                                  type="button"
                                  onClick={() => toggleCalendarHour(row.broadcast_date, hour)}
                                >
                                  {hour}
                                </button>
                              );
                            })}
                          </div>
                        </section>
                      ))}
                    </div>
                  )}
                </div>
              )
            )}

            <div className="form-actions">
              <button className="ghost-button" type="button" onClick={() => setHoursModal(null)}>Cancel</button>
              <button className="primary-button" disabled={hoursSaving || hoursLoading} type="button" onClick={saveBroadcastHours}>
                {hoursSaving ? 'Saving...' : 'Save'}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  );
}

function hourKey(day, hour) {
  return `${day}:${hour}`;
}

function parseHourKey(key) {
  const [iso_weekday, hour] = key.split(':').map(Number);
  return { iso_weekday, hour };
}

function allHourKeys() {
  const keys = [];
  for (const day of DAYS) {
    for (const hour of HOURS) {
      keys.push(hourKey(day.id, hour));
    }
  }
  return keys;
}

function groupCalendarHours(hours) {
  const map = new Map();
  for (const slot of hours) {
    if (slot.active === false) continue;
    const broadcastDate = slot.broadcast_date;
    if (!broadcastDate) continue;
    const hour = Number(slot.hour);
    if (!Number.isInteger(hour) || hour < 0 || hour > 23) continue;
    if (!map.has(broadcastDate)) map.set(broadcastDate, new Set());
    map.get(broadcastDate).add(hour);
  }

  return [...map.entries()]
    .map(([broadcast_date, hoursSet]) => ({ broadcast_date, hours: hoursSet }))
    .sort((a, b) => a.broadcast_date.localeCompare(b.broadcast_date));
}

function calendarDatesToPayload(calendarDates) {
  return calendarDates
    .filter((row) => row.hours.size > 0)
    .map((row) => ({
      broadcast_date: row.broadcast_date,
      hours: [...row.hours].sort((a, b) => a - b)
    }));
}

function calendarHoursCount(calendarDates) {
  return calendarDates.reduce((total, row) => total + row.hours.size, 0);
}

function localDateToday() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
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
