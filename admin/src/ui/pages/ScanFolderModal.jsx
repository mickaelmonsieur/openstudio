import { useEffect, useState } from 'react';

export function ScanFolderModal({ genres, subcategories, onClose, onFinished }) {
  const [root, setRoot] = useState(null);
  const [childrenByPath, setChildrenByPath] = useState({});
  const [expanded, setExpanded] = useState({});
  const [selectedPath, setSelectedPath] = useState('');
  const [genreId, setGenreId] = useState('');
  const [subcategoryId, setSubcategoryId] = useState('');
  const [includeSubfolders, setIncludeSubfolders] = useState(true);
  const [loadingPath, setLoadingPath] = useState('');
  const [error, setError] = useState(null);
  const [job, setJob] = useState(null);
  const [notified, setNotified] = useState(false);

  const grouped = groupByCategory(subcategories);
  const running = job && !['completed', 'failed'].includes(job.status);
  const progress = job?.total ? Math.round((job.processed / job.total) * 100) : 0;

  useEffect(() => {
    loadFolder('');
  }, []);

  useEffect(() => {
    if (!job || ['completed', 'failed'].includes(job.status)) return undefined;

    const timer = setInterval(async () => {
      try {
        const payload = await fetchJson(`/api/tracks/folder-import/${job.id}`);
        setJob(payload.job);
      } catch (err) {
        setError(err.message);
      }
    }, 500);

    return () => clearInterval(timer);
  }, [job?.id, job?.status]);

  useEffect(() => {
    if (job?.status === 'completed' && !notified) {
      setNotified(true);
      onFinished();
    }
  }, [job?.status, notified, onFinished]);

  async function loadFolder(folderPath) {
    setLoadingPath(folderPath || 'root');
    setError(null);
    try {
      const params = new URLSearchParams();
      if (folderPath) params.set('path', folderPath);
      const payload = await fetchJson(`/api/tracks/folders?${params.toString()}`);

      setRoot((prev) => prev || payload.folder);
      setChildrenByPath((prev) => ({
        ...prev,
        [payload.folder.path]: payload.children || []
      }));
      setExpanded((prev) => ({
        ...prev,
        [payload.folder.path]: true
      }));
      setSelectedPath((prev) => prev || payload.folder.path);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoadingPath('');
    }
  }

  function toggleFolder(node) {
    if (!node.canOpen) return;
    if (!childrenByPath[node.path]) {
      loadFolder(node.path);
      return;
    }

    setExpanded((prev) => ({
      ...prev,
      [node.path]: !prev[node.path]
    }));
  }

  async function startScan(event) {
    event.preventDefault();
    setError(null);
    setNotified(false);

    try {
      const payload = await fetchJson('/api/tracks/folder-import', {
        method: 'POST',
        body: JSON.stringify({
          folderPath: selectedPath,
          genre_id: genreId,
          subcategory_id: subcategoryId,
          includeSubfolders
        })
      });
      setJob(payload.job);
    } catch (err) {
      setError(err.message);
    }
  }

  return (
    <div className="modal-backdrop">
      <section className="modal-panel scan-modal" role="dialog" aria-modal="true">
        <header className="modal-header">
          <div>
            <p className="panel-kicker">Import</p>
            <h2>Scan Folder</h2>
          </div>
          <button className="icon-button" onClick={onClose} type="button">×</button>
        </header>

        {!job ? (
          <form className="scan-layout" onSubmit={startScan}>
            <div className="folder-tree-panel">
              <div className="folder-tree">
                {root ? renderFolderNode(root, {
                  childrenByPath,
                  expanded,
                  loadingPath,
                  selectedPath,
                  onSelect: setSelectedPath,
                  onToggle: toggleFolder
                }) : <div className="table-loading">Loading...</div>}
              </div>
            </div>

            <div className="scan-settings">
              <label>
                <span>Selected Folder</span>
                <input readOnly type="text" value={selectedPath} />
              </label>

              <label>
                <span>Genre</span>
                <select required value={genreId} onChange={(e) => setGenreId(e.target.value)}>
                  <option value="">Select a genre...</option>
                  {genres.map((genre) => (
                    <option key={genre.id} value={genre.id}>{genre.name}</option>
                  ))}
                </select>
              </label>

              <label>
                <span>Category</span>
                <select required value={subcategoryId} onChange={(e) => setSubcategoryId(e.target.value)}>
                  <option value="">Select a category...</option>
                  {grouped.map((group) => (
                    <optgroup key={group.category} label={group.category}>
                      {group.items.map((subcategory) => (
                        <option key={subcategory.id} value={subcategory.id}>{subcategory.name}</option>
                      ))}
                    </optgroup>
                  ))}
                </select>
              </label>

              <label className="checkbox-field">
                <span>Include Subfolders</span>
                <input
                  checked={includeSubfolders}
                  type="checkbox"
                  onChange={(e) => setIncludeSubfolders(e.target.checked)}
                />
              </label>

              {error ? <div className="form-error">{error}</div> : null}

              <div className="form-actions">
                <button className="ghost-button" onClick={onClose} type="button">Cancel</button>
                <button className="primary-button" disabled={!selectedPath} type="submit">
                  Start Scan
                </button>
              </div>
            </div>
          </form>
        ) : (
          <div className="scan-progress">
            <div className="progress-header">
              <strong>{job.status === 'scanning' ? 'Scanning...' : 'Importing...'}</strong>
              <span>{job.processed} / {job.total}</span>
            </div>
            <div className="progress-bar">
              <div style={{ width: `${progress}%` }} />
            </div>

            <div className="scan-counters">
              <span>Created: <strong>{job.created}</strong></span>
              <span>Skipped: <strong>{job.skipped}</strong></span>
              <span>Errors: <strong>{job.errors}</strong></span>
            </div>

            {job.current ? (
              <div className="import-summary">
                <span>{job.current}</span>
              </div>
            ) : null}

            {error ? <div className="form-error">{error}</div> : null}

            <div className="job-messages">
              {job.messages.map((entry, index) => (
                <div key={`${entry.at}-${index}`}>{entry.message}</div>
              ))}
            </div>

            <div className="form-actions">
              <button className="ghost-button" onClick={onClose} type="button">
                {running ? 'Hide' : 'Close'}
              </button>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

function renderFolderNode(node, context) {
  const children = context.childrenByPath[node.path] || [];
  const isExpanded = Boolean(context.expanded[node.path]);
  const isSelected = context.selectedPath === node.path;
  const isLoading = context.loadingPath === node.path;

  return (
    <div className="folder-node" key={node.path}>
      <div className={`folder-row ${isSelected ? 'selected' : ''}`}>
        <button
          className="folder-toggle"
          disabled={!node.canOpen}
          type="button"
          onClick={() => context.onToggle(node)}
        >
          {node.canOpen ? (isExpanded ? '▾' : '▸') : ''}
        </button>
        <button className="folder-name" type="button" onClick={() => context.onSelect(node.path)}>
          {node.name}
        </button>
      </div>

      {isLoading ? <div className="folder-loading">Loading...</div> : null}

      {isExpanded && children.length > 0 ? (
        <div className="folder-children">
          {children.map((child) => renderFolderNode(child, context))}
        </div>
      ) : null}
    </div>
  );
}

function groupByCategory(subcategories) {
  const map = new Map();
  for (const subcategory of subcategories) {
    if (!map.has(subcategory.category_name)) map.set(subcategory.category_name, []);
    map.get(subcategory.category_name).push(subcategory);
  }
  return Array.from(map.entries()).map(([category, items]) => ({ category, items }));
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
