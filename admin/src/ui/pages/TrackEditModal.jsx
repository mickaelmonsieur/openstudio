import { useState } from 'react';

export function TrackEditModal({ track, artists, subcategories, error, saving, onClose, onSubmit }) {
  const [formData, setFormData] = useState({
    artist_id:      track.artist_id      ?? '',
    title:          track.title          ?? '',
    album:          track.album          ?? '',
    year:           track.year           ?? '',
    subcategory_id: track.subcategory_id ?? '',
    active:         Boolean(track.active)
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
      <section className="modal-panel" role="dialog" aria-modal="true">
        <header className="modal-header">
          <div>
            <p className="panel-kicker">Edit</p>
            <h2>Track</h2>
          </div>
          <button className="icon-button" onClick={onClose} type="button">×</button>
        </header>

        <form className="resource-form" onSubmit={submit}>
          <label>
            <span>Artist</span>
            <select
              value={formData.artist_id}
              onChange={(e) => update('artist_id', e.target.value)}
            >
              <option value="">— None —</option>
              {artists.map((a) => (
                <option key={a.id} value={a.id}>{a.name}</option>
              ))}
            </select>
          </label>

          <label>
            <span>Title</span>
            <input
              maxLength={64}
              required
              type="text"
              value={formData.title}
              onChange={(e) => update('title', e.target.value)}
            />
          </label>

          <label>
            <span>Album</span>
            <input
              maxLength={64}
              type="text"
              value={formData.album}
              onChange={(e) => update('album', e.target.value)}
            />
          </label>

          <label>
            <span>Year</span>
            <input
              max={2100}
              min={1900}
              type="number"
              value={formData.year}
              onChange={(e) => update('year', e.target.value)}
            />
          </label>

          <label>
            <span>Category</span>
            <select
              value={formData.subcategory_id}
              onChange={(e) => update('subcategory_id', e.target.value)}
            >
              <option value="">— None —</option>
              {grouped.map((group) => (
                <optgroup key={group.category} label={group.category}>
                  {group.items.map((sc) => (
                    <option key={sc.id} value={sc.id}>{sc.name}</option>
                  ))}
                </optgroup>
              ))}
            </select>
          </label>

          <label className="checkbox-field">
            <span>Active</span>
            <input
              checked={formData.active}
              type="checkbox"
              onChange={(e) => update('active', e.target.checked)}
            />
          </label>

          {error ? <div className="form-error">{error}</div> : null}

          <div className="form-actions">
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

function groupByCategory(subcategories) {
  const map = new Map();
  for (const sc of subcategories) {
    if (!map.has(sc.category_name)) map.set(sc.category_name, []);
    map.get(sc.category_name).push(sc);
  }
  return Array.from(map.entries()).map(([category, items]) => ({ category, items }));
}
