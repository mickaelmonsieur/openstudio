import { useState } from 'react';

const GENDER_OPTIONS  = [{ value: 0, label: 'Male' }, { value: 1, label: 'Female' }, { value: 2, label: 'Mixed' }];
const START_END_TYPES = [{ value: 0, label: 'Soft' }, { value: 1, label: 'Hard' }, { value: 2, label: 'Fade' }, { value: 3, label: 'Voice' }];

export function TrackEditModal({ mode = 'edit', track, artists, genres, subcategories, trackTypes = [], moods = [], languages = [], error, saving, onClose, onSubmit }) {
  const [tab, setTab] = useState('general');
  const [formData, setFormData] = useState({
    artist_id:      track.artist_id      ?? '',
    genre_id:       track.genre_id       ?? '',
    title:          track.title          ?? '',
    album:          track.album          ?? '',
    year:           track.year           ?? '',
    duration:       track.duration       ?? 0,
    sample_rate:    track.sample_rate    ?? 44100,
    path:           track.path           ?? '',
    subcategory_id: track.subcategory_id ?? '',
    active:         track.active         ?? true,
    cue_in:         track.cue_in         ?? 0,
    cue_out:        track.cue_out        ?? null,
    start_date:     track.start_date     ? track.start_date.slice(0, 10) : '',
    end_date:       track.end_date       ? track.end_date.slice(0, 10)   : '',
    priority:       track.priority       ?? 0,
    track_type_id:  track.track_type_id  ?? '',
    mood_id:        track.mood_id        ?? '',
    language:       track.language       ?? '',
    gender:         track.gender         ?? '',
    start_type:     track.start_type     ?? '',
    end_type:       track.end_type       ?? '',
    comment:        track.comment        ?? ''
  });

  const grouped = groupByCategory(subcategories);

  function update(key, value) {
    setFormData((prev) => ({ ...prev, [key]: value }));
  }

  function submit(event) {
    event.preventDefault();
    onSubmit(formData);
  }

  return (
    <div className="modal-backdrop">
      <section className="modal-panel track-edit-modal" role="dialog" aria-modal="true">
        <header className="modal-header">
          <div>
            <p className="panel-kicker">{mode === 'create' ? 'Add' : 'Edit'}</p>
            <h2>Track</h2>
          </div>
          <button className="icon-button" onClick={onClose} type="button">×</button>
        </header>

        <div className="modal-tabs">
          <button className={`modal-tab${tab === 'general' ? ' modal-tab--active' : ''}`} type="button" onClick={() => setTab('general')}>General</button>
          <button className={`modal-tab${tab === 'details' ? ' modal-tab--active' : ''}`} type="button" onClick={() => setTab('details')}>Details</button>
        </div>

        <form className="resource-form event-form-grid" onSubmit={submit}>

          {/* ── General tab ── */}
          {tab === 'general' ? (
            <>
              <div className="event-col-left">
                <label>
                  <span>Artist</span>
                  <select value={formData.artist_id} onChange={(e) => update('artist_id', e.target.value)}>
                    <option value="">— None —</option>
                    {artists.map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
                  </select>
                </label>

                <label>
                  <span>Title</span>
                  <input maxLength={64} required type="text" value={formData.title} onChange={(e) => update('title', e.target.value)} />
                </label>

                <label>
                  <span>Album</span>
                  <input maxLength={64} type="text" value={formData.album} onChange={(e) => update('album', e.target.value)} />
                </label>

                <div className="form-row">
                  <label>
                    <span>Year</span>
                    <input max={2100} min={1900} type="number" value={formData.year} onChange={(e) => update('year', e.target.value)} />
                  </label>
                  <label>
                    <span>Priority</span>
                    <input type="number" value={formData.priority} onChange={(e) => update('priority', Number(e.target.value))} />
                  </label>
                </div>

                <label className="checkbox-field">
                  <span>Active</span>
                  <input checked={formData.active} type="checkbox" onChange={(e) => update('active', e.target.checked)} />
                </label>

                {mode === 'create' ? (
                  <div className="import-summary">
                    <span>{formData.path}</span>
                    <strong>{formatDuration(formData.duration)} - {formData.sample_rate || 44100} Hz</strong>
                  </div>
                ) : null}
              </div>

              <div className="event-col-right">
                <label>
                  <span>Genre</span>
                  <select value={formData.genre_id} onChange={(e) => update('genre_id', e.target.value)}>
                    <option value="">— None —</option>
                    {genres.map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
                  </select>
                </label>

                <label>
                  <span>Category</span>
                  <select required value={formData.subcategory_id} onChange={(e) => update('subcategory_id', e.target.value)}>
                    <option value="">Select a category...</option>
                    {grouped.map((group) =>
                      group.items.length === 1 && group.items[0].name === group.category ? (
                        <option key={group.items[0].id} value={group.items[0].id}>{group.category}</option>
                      ) : (
                        <optgroup key={group.category} label={group.category}>
                          {group.items.map((sc) => <option key={sc.id} value={sc.id}>{sc.name}</option>)}
                        </optgroup>
                      )
                    )}
                  </select>
                </label>

                <label>
                  <span>Type</span>
                  <select value={formData.track_type_id} onChange={(e) => update('track_type_id', e.target.value)}>
                    <option value="">— None —</option>
                    {trackTypes.map((t) => <option key={t.id} value={t.id}>{t.name}</option>)}
                  </select>
                </label>

                <div className="form-row">
                  <label>
                    <span>Start date</span>
                    <input type="date" value={formData.start_date} onChange={(e) => update('start_date', e.target.value)} />
                  </label>
                  <label>
                    <span>End date</span>
                    <input type="date" value={formData.end_date} onChange={(e) => update('end_date', e.target.value)} />
                  </label>
                </div>
              </div>
            </>
          ) : null}

          {/* ── Details tab ── */}
          {tab === 'details' ? (
            <>
              <div className="event-col-left">
                <label>
                  <span>Mood</span>
                  <select value={formData.mood_id} onChange={(e) => update('mood_id', e.target.value)}>
                    <option value="">— None —</option>
                    {moods.map((m) => <option key={m.id} value={m.id}>{m.name}</option>)}
                  </select>
                </label>

                <label>
                  <span>Language</span>
                  <select value={formData.language} onChange={(e) => update('language', e.target.value)}>
                    <option value="">— None —</option>
                    {languages.map((l) => <option key={l.alpha2} value={l.alpha2}>{l.name}</option>)}
                  </select>
                </label>

                <label>
                  <span>Gender</span>
                  <select value={formData.gender} onChange={(e) => update('gender', e.target.value === '' ? '' : Number(e.target.value))}>
                    <option value="">— None —</option>
                    {GENDER_OPTIONS.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                  </select>
                </label>
              </div>

              <div className="event-col-right">
                <div className="form-row">
                  <label>
                    <span>Start type</span>
                    <select value={formData.start_type} onChange={(e) => update('start_type', e.target.value === '' ? '' : Number(e.target.value))}>
                      <option value="">— None —</option>
                      {START_END_TYPES.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                    </select>
                  </label>
                  <label>
                    <span>End type</span>
                    <select value={formData.end_type} onChange={(e) => update('end_type', e.target.value === '' ? '' : Number(e.target.value))}>
                      <option value="">— None —</option>
                      {START_END_TYPES.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                    </select>
                  </label>
                </div>

                <label>
                  <span>Comment</span>
                  <textarea rows={4} value={formData.comment} onChange={(e) => update('comment', e.target.value)} />
                </label>
              </div>
            </>
          ) : null}

          {error ? <div className="form-error event-form-full">{error}</div> : null}

          <div className="form-actions event-form-full">
            <button className="ghost-button" onClick={onClose} type="button">Cancel</button>
            <button className="primary-button" disabled={saving} type="submit">
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

function formatDuration(seconds) {
  const value = Number(seconds || 0);
  const total = Math.max(0, Math.round(value));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function groupByCategory(subcategories) {
  const map = new Map();
  for (const sc of subcategories) {
    if (!map.has(sc.category_name)) map.set(sc.category_name, []);
    map.get(sc.category_name).push(sc);
  }
  return Array.from(map.entries()).map(([category, items]) => ({ category, items }));
}
