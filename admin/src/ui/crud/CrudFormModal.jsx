import { useEffect, useState } from 'react';

export function CrudFormModal({ mode, resource, row, error, saving, onClose, onSubmit }) {
  const [formData, setFormData] = useState(() => initialData(resource, row));

  useEffect(() => {
    setFormData(initialData(resource, row));
  }, [resource, row]);

  function updateField(field, value) {
    setFormData((current) => ({
      ...current,
      [field]: value
    }));
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
            <p className="panel-kicker">{mode === 'create' ? 'Add' : 'Edit'}</p>
            <h2>{resource.title}</h2>
          </div>
          <button className="icon-button" onClick={onClose} type="button">
            ×
          </button>
        </header>

        <form className="resource-form" onSubmit={submit}>
          {resource.form.map((field) => (
            <label className={field.type === 'checkbox' ? 'checkbox-field' : ''} key={field.key}>
              <span>{field.label}</span>
              {field.type === 'checkbox' ? (
                <input
                  checked={Boolean(formData[field.key])}
                  type="checkbox"
                  onChange={(event) => updateField(field.key, event.target.checked)}
                />
              ) : (
                <input
                  maxLength={field.maxLength}
                  required={field.required}
                  type={field.type || 'text'}
                  value={formData[field.key] ?? ''}
                  onChange={(event) => updateField(field.key, event.target.value)}
                />
              )}
            </label>
          ))}

          {error ? <div className="form-error">{error}</div> : null}

          <div className="form-actions">
            <button className="ghost-button" onClick={onClose} type="button">
              Cancel
            </button>
            <button className="primary-button" disabled={saving} type="submit">
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

function initialData(resource, row) {
  const data = {};
  for (const field of resource.form) {
    data[field.key] = row?.[field.key] ?? (field.type === 'checkbox' ? false : '');
  }
  return data;
}
