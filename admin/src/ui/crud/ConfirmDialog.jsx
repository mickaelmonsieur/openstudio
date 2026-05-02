export function ConfirmDialog({ title, message, busy, onCancel, onConfirm }) {
  return (
    <div className="modal-backdrop">
      <section className="modal-panel confirm-panel" role="dialog" aria-modal="true">
        <header className="modal-header">
          <h2>{title}</h2>
        </header>
        <p>{message}</p>
        <div className="form-actions">
          <button className="ghost-button" onClick={onCancel} type="button">
            Cancel
          </button>
          <button className="danger-button solid" disabled={busy} onClick={onConfirm} type="button">
            {busy ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      </section>
    </div>
  );
}
