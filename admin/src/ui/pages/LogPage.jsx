import { useEffect, useState } from 'react';

export function LogPage({ resource }) {
  const [rows, setRows] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const limit = resource.pageSize ?? 100;
  const totalPages = Math.max(1, Math.ceil(total / limit));

  useEffect(() => {
    setPage(1);
  }, [resource.endpoint]);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setError(null);
      try {
        const response = await fetch(`${resource.endpoint}?page=${page}&limit=${limit}`);
        const payload = await response.json();
        if (!response.ok) throw new Error(payload.error || `Error ${response.status}`);
        if (!cancelled) {
          setRows(payload.rows || []);
          setTotal(payload.total || 0);
        }
      } catch (err) {
        if (!cancelled) setError(err.message);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();
    return () => { cancelled = true; };
  }, [resource.endpoint, page, limit]);

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Log</p>
          <h2>{resource.title}</h2>
        </div>
        <span className="log-total">{total.toLocaleString()} entries</span>
      </header>

      {error ? <div className="table-error">{error}</div> : null}

      {loading ? (
        <div className="table-loading">Loading...</div>
      ) : (
        <>
          <div className="data-table-wrap">
            <table className="data-table">
              <thead>
                <tr>
                  {resource.columns.map((col) => (
                    <th key={col.key} style={{ width: col.width }}>{col.label}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.length === 0 ? (
                  <tr>
                    <td className="empty-cell" colSpan={resource.columns.length}>No records.</td>
                  </tr>
                ) : (
                  rows.map((row) => (
                    <tr key={row.id}>
                      {resource.columns.map((col) => (
                        <td key={col.key}>{formatCell(row[col.key])}</td>
                      ))}
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <div className="pagination">
            <button
              className="ghost-button"
              disabled={page <= 1}
              onClick={() => setPage((p) => p - 1)}
              type="button"
            >
              ← Prev
            </button>
            <span className="pagination-info">
              Page {page} of {totalPages} &mdash; {total.toLocaleString()} total
            </span>
            <button
              className="ghost-button"
              disabled={page >= totalPages}
              onClick={() => setPage((p) => p + 1)}
              type="button"
            >
              Next →
            </button>
          </div>
        </>
      )}
    </section>
  );
}

function formatCell(value) {
  if (value === null || value === undefined || value === '') return '—';
  if (typeof value === 'boolean') return value ? 'Yes' : 'No';
  return String(value);
}
