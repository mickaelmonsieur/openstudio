export function ImportPage() {
  function downloadDump() {
    window.location.href = '/api/database/export';
  }

  return (
    <section className="crud-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Admin</p>
          <h2>Import / Export</h2>
        </div>
      </header>

      <section className="playlist-panel">
        <div className="coverage-header">
          <div>
            <p className="panel-kicker">Export</p>
            <h2>Download Database Dump</h2>
          </div>
        </div>

        <p style={{ color: 'var(--color-text-muted, #888)', marginBottom: '1.5rem' }}>
          Exports the full database as plain SQL INSERT statements, compressed with gzip
          (<code>.sql.gz</code>). The dump can be restored on any PostgreSQL instance
          compatible with the OpenStudio schema.
        </p>

        <div className="form-actions">
          <button className="primary-button" type="button" onClick={downloadDump}>
            Download DB Dump (.sql.gz)
          </button>
        </div>
      </section>
    </section>
  );
}
