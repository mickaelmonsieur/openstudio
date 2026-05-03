import { useEffect, useMemo, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { DataTable } from '../crud/DataTable.jsx';

const TEMPLATE_COLS = [
  { key: 'name', label: 'Name' }
];

const SLOT_COLS = [
  { key: 'category_name',    label: 'Category',     width: '130px' },
  { key: 'subcategory_name', label: 'Subcategory',  width: '130px' },
  { key: 'comment',          label: 'Comment' }
];

function emptySlotForm() {
  return { category_id: '', subcategory_id: '', comment: '', track_protection: 3600, artist_protection: 3600 };
}

export function TemplatesPage() {
  const [templates, setTemplates]               = useState([]);
  const [slots, setSlots]                       = useState([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState(null);
  const [categories, setCategories]             = useState([]);
  const [subcategories, setSubcategories]       = useState([]);
  const [loadingTemplates, setLoadingTemplates] = useState(true);
  const [loadingSlots, setLoadingSlots]         = useState(false);
  const [error, setError]                       = useState(null);
  const [modal, setModal]                       = useState(null);
  const [templateName, setTemplateName]         = useState('');
  const [slotForm, setSlotForm]                 = useState(emptySlotForm);
  const [formError, setFormError]               = useState(null);
  const [saving, setSaving]                     = useState(false);
  const [orderingSlots, setOrderingSlots]       = useState(false);
  const [deleteTarget, setDeleteTarget]         = useState(null);

  const selectedTemplate = useMemo(
    () => templates.find((t) => t.id === selectedTemplateId) || null,
    [templates, selectedTemplateId]
  );

  const filteredSubcategories = useMemo(
    () => subcategories.filter((s) => s.category_id === Number(slotForm.category_id)),
    [subcategories, slotForm.category_id]
  );

  const displaySlots = useMemo(
    () => slots,
    [slots]
  );

  useEffect(() => {
    loadTemplates();
    loadOptions();
  }, []);

  useEffect(() => {
    if (!selectedTemplateId) { setSlots([]); return; }
    loadSlots(selectedTemplateId);
  }, [selectedTemplateId]);

  async function loadTemplates(nextSelectedId = selectedTemplateId) {
    setLoadingTemplates(true);
    setError(null);
    try {
      const payload = await fetchJson('/api/templates');
      const rows = payload.rows || [];
      setTemplates(rows);
      if (rows.length === 0) {
        setSelectedTemplateId(null);
      } else if (nextSelectedId && rows.some((r) => r.id === nextSelectedId)) {
        setSelectedTemplateId(nextSelectedId);
      } else {
        setSelectedTemplateId(rows[0].id);
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoadingTemplates(false);
    }
  }

  async function loadOptions() {
    try {
      const opts = await fetchJson('/api/template-slots/options');
      setCategories(opts.categories || []);
      setSubcategories(opts.subcategories || []);
    } catch {
      // non-critical
    }
  }

  async function loadSlots(templateId) {
    setLoadingSlots(true);
    setError(null);
    try {
      const payload = await fetchJson(`/api/templates/${templateId}/slots`);
      setSlots(payload.rows || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoadingSlots(false);
    }
  }

  function openCreateTemplate() {
    setTemplateName('');
    setFormError(null);
    setModal({ kind: 'template', mode: 'create', row: null });
  }

  function openEditTemplate(row) {
    setTemplateName(row.name);
    setFormError(null);
    setModal({ kind: 'template', mode: 'edit', row });
  }

  function openCreateSlot(insertAfterRow = null) {
    if (!selectedTemplate) return;
    setSlotForm(emptySlotForm());
    setFormError(null);
    setModal({
      kind: 'slot',
      mode: 'create',
      row: null,
      insertAfterId: insertAfterRow?.id || null
    });
  }

  function openEditSlot(row) {
    setSlotForm({
      category_id:       String(row.category_id ?? ''),
      subcategory_id:    row.subcategory_id ? String(row.subcategory_id) : '',
      comment:           row.comment || '',
      track_protection:  row.track_protection ?? 3600,
      artist_protection: row.artist_protection ?? 3600
    });
    setFormError(null);
    setModal({ kind: 'slot', mode: 'edit', row });
  }

  async function saveModal(event) {
    event.preventDefault();
    if (!modal) return;
    setSaving(true);
    setFormError(null);
    try {
      if (modal.kind === 'template') {
        const editing = modal.mode === 'edit';
        const url = editing ? `/api/templates/${modal.row.id}` : '/api/templates';
        const payload = await fetchJson(url, {
          method: editing ? 'PUT' : 'POST',
          body: JSON.stringify({ name: templateName })
        });
        setModal(null);
        await loadTemplates(payload.row.id);
      } else {
        const editing = modal.mode === 'edit';
        const url = editing
          ? `/api/template-slots/${modal.row.id}`
          : `/api/templates/${selectedTemplate.id}/slots`;
        const body = editing
          ? slotForm
          : { ...slotForm, insert_after_id: modal.insertAfterId };
        await fetchJson(url, {
          method: editing ? 'PUT' : 'POST',
          body: JSON.stringify(body)
        });
        setModal(null);
        await loadSlots(selectedTemplateId);
      }
    } catch (err) {
      setFormError(err.message);
    } finally {
      setSaving(false);
    }
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    setSaving(true);
    setError(null);
    try {
      if (deleteTarget.kind === 'template') {
        await fetchJson(`/api/templates/${deleteTarget.row.id}`, { method: 'DELETE' });
        setDeleteTarget(null);
        await loadTemplates();
      } else {
        await fetchJson(`/api/template-slots/${deleteTarget.row.id}`, { method: 'DELETE' });
        setDeleteTarget(null);
        await loadSlots(selectedTemplateId);
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  }

  async function reorderSlots(nextRows) {
    if (!selectedTemplateId || orderingSlots) return;

    const previousSlots = slots;
    const slotsById = new Map(slots.map((slot) => [slot.id, slot]));
    const nextSlots = nextRows.map((row, index) => ({
      ...(slotsById.get(row.id) || row),
      position: index + 1
    }));
    setSlots(nextSlots);
    setOrderingSlots(true);
    setError(null);

    try {
      const payload = await fetchJson(`/api/templates/${selectedTemplateId}/slots/order`, {
        method: 'PUT',
        body: JSON.stringify({ ids: nextSlots.map((row) => row.id) })
      });
      setSlots(payload.rows || nextSlots);
    } catch (err) {
      setSlots(previousSlots);
      setError(err.message);
    } finally {
      setOrderingSlots(false);
    }
  }

  function moveSlot(row, direction) {
    if (orderingSlots) return;

    const fromIndex = slots.findIndex((slot) => slot.id === row.id);
    const toIndex = fromIndex + direction;
    if (fromIndex < 0 || toIndex < 0 || toIndex >= slots.length) return;

    const nextSlots = [...slots];
    const [moved] = nextSlots.splice(fromIndex, 1);
    nextSlots.splice(toIndex, 0, moved);
    reorderSlots(nextSlots);
  }

  return (
    <section className="category-page">
      {error ? <div className="table-error">{error}</div> : null}

      <section className="split-crud-panel">
        <header className="crud-header">
          <div>
            <p className="panel-kicker">Master</p>
            <h2>Templates</h2>
          </div>
          <button className="primary-button" onClick={openCreateTemplate} type="button">Add</button>
        </header>

        {loadingTemplates ? (
          <div className="table-loading">Loading...</div>
        ) : (
          <DataTable
            columns={TEMPLATE_COLS}
            primaryKey="id"
            rows={templates}
            selectedId={selectedTemplateId}
            onDelete={(row) => setDeleteTarget({ kind: 'template', row })}
            onEdit={openEditTemplate}
            onSelect={(row) => setSelectedTemplateId(row.id)}
          />
        )}
      </section>

      <section className="split-crud-panel">
        <header className="crud-header">
          <div>
            <p className="panel-kicker">Detail</p>
            <h2>{selectedTemplate ? `${selectedTemplate.name} — Slots` : 'Slots'}</h2>
          </div>
          <button
            className="primary-button"
            disabled={!selectedTemplate}
            onClick={openCreateSlot}
            type="button"
          >
            Add
          </button>
        </header>

        {!selectedTemplate ? (
          <div className="table-loading">Select a template.</div>
        ) : loadingSlots ? (
          <div className="table-loading">Loading...</div>
        ) : (
          <DataTable
            columns={SLOT_COLS}
            onReorder={reorderSlots}
            primaryKey="id"
            renderRowActions={(row, index) => (
              <>
                <button
                  aria-label="Add slot below"
                  className="ghost-button table-icon-button"
                  disabled={orderingSlots}
                  onClick={(event) => {
                    event.stopPropagation();
                    openCreateSlot(row);
                  }}
                  title="Add below"
                  type="button"
                >
                  <i className="bi bi-plus-lg" aria-hidden="true" />
                </button>
                <button
                  aria-label="Move slot up"
                  className="ghost-button table-icon-button"
                  disabled={orderingSlots || index === 0}
                  onClick={(event) => {
                    event.stopPropagation();
                    moveSlot(row, -1);
                  }}
                  title="Move up"
                  type="button"
                >
                  <i className="bi bi-arrow-up" aria-hidden="true" />
                </button>
                <button
                  aria-label="Move slot down"
                  className="ghost-button table-icon-button"
                  disabled={orderingSlots || index === displaySlots.length - 1}
                  onClick={(event) => {
                    event.stopPropagation();
                    moveSlot(row, 1);
                  }}
                  title="Move down"
                  type="button"
                >
                  <i className="bi bi-arrow-down" aria-hidden="true" />
                </button>
              </>
            )}
            reorderable
            rows={displaySlots}
            onDelete={(row) => setDeleteTarget({ kind: 'slot', row })}
            onEdit={openEditSlot}
          />
        )}
      </section>

      {modal ? (
        <div className="modal-backdrop">
          <section className="modal-panel" role="dialog" aria-modal="true">
            {modal.kind === 'template' ? (
              <>
                <header className="modal-header">
                  <div>
                    <p className="panel-kicker">{modal.mode === 'create' ? 'Add' : 'Edit'}</p>
                    <h2>Template</h2>
                  </div>
                  <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
                </header>
                <form className="resource-form" onSubmit={saveModal}>
                  <label>
                    <span>Name</span>
                    <input
                      autoFocus
                      maxLength={32}
                      required
                      type="text"
                      value={templateName}
                      onChange={(e) => setTemplateName(e.target.value)}
                    />
                  </label>
                  {formError ? <div className="form-error">{formError}</div> : null}
                  <div className="form-actions">
                    <button className="ghost-button" type="button" onClick={() => setModal(null)}>Cancel</button>
                    <button className="primary-button" disabled={saving} type="submit">
                      {saving ? 'Saving...' : 'Save'}
                    </button>
                  </div>
                </form>
              </>
            ) : (
              <>
                <header className="modal-header">
                  <div>
                    <p className="panel-kicker">{modal.mode === 'create' ? 'Add' : 'Edit'}</p>
                    <h2>Slot</h2>
                  </div>
                  <button className="icon-button" type="button" onClick={() => setModal(null)}>×</button>
                </header>
                <form className="resource-form" onSubmit={saveModal}>
                  <label>
                    <span>Category</span>
                    <select
                      required
                      value={slotForm.category_id}
                      onChange={(e) => setSlotForm((prev) => ({ ...prev, category_id: e.target.value, subcategory_id: '' }))}
                    >
                      <option value="">— select —</option>
                      {categories.map((c) => (
                        <option key={c.id} value={c.id}>{c.name}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    <span>Subcategory</span>
                    <select
                      value={slotForm.subcategory_id}
                      onChange={(e) => setSlotForm((prev) => ({ ...prev, subcategory_id: e.target.value }))}
                    >
                      <option value="">— none —</option>
                      {filteredSubcategories.map((s) => (
                        <option key={s.id} value={s.id}>{s.name}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    <span>Comment</span>
                    <input
                      maxLength={64}
                      type="text"
                      value={slotForm.comment}
                      onChange={(e) => setSlotForm((prev) => ({ ...prev, comment: e.target.value }))}
                    />
                  </label>
                  <div className="form-row">
                    <label>
                      <span>Track Prot. (s)</span>
                      <input
                        min="0"
                        required
                        type="number"
                        value={slotForm.track_protection}
                        onChange={(e) => setSlotForm((prev) => ({ ...prev, track_protection: Number(e.target.value) }))}
                      />
                    </label>
                    <label>
                      <span>Artist Prot. (s)</span>
                      <input
                        min="0"
                        required
                        type="number"
                        value={slotForm.artist_protection}
                        onChange={(e) => setSlotForm((prev) => ({ ...prev, artist_protection: Number(e.target.value) }))}
                      />
                    </label>
                  </div>
                  {formError ? <div className="form-error">{formError}</div> : null}
                  <div className="form-actions">
                    <button className="ghost-button" type="button" onClick={() => setModal(null)}>Cancel</button>
                    <button className="primary-button" disabled={saving} type="submit">
                      {saving ? 'Saving...' : 'Save'}
                    </button>
                  </div>
                </form>
              </>
            )}
          </section>
        </div>
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={saving}
          message={
            deleteTarget.kind === 'template'
              ? `Delete template "${deleteTarget.row.name}"?`
              : `Delete this slot?`
          }
          title={`Delete ${deleteTarget.kind === 'template' ? 'Template' : 'Slot'}`}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) },
    ...options
  });
  if (response.status === 204) return {};
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
  return payload;
}
