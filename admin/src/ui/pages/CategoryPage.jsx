import { useEffect, useMemo, useState } from 'react';
import { ConfirmDialog } from '../crud/ConfirmDialog.jsx';
import { CrudFormModal } from '../crud/CrudFormModal.jsx';
import { DataTable } from '../crud/DataTable.jsx';
import { categoriesResource, subcategoriesResource } from '../resources/categories.js';

export function CategoryPage() {
  const [categories, setCategories] = useState([]);
  const [subcategories, setSubcategories] = useState([]);
  const [selectedCategoryId, setSelectedCategoryId] = useState(null);
  const [loadingCategories, setLoadingCategories] = useState(true);
  const [loadingSubcategories, setLoadingSubcategories] = useState(false);
  const [error, setError] = useState(null);
  const [modal, setModal] = useState(null);
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState(null);

  const selectedCategory = useMemo(() => {
    return categories.find((category) => category.id === selectedCategoryId) || null;
  }, [categories, selectedCategoryId]);

  useEffect(() => {
    loadCategories();
  }, []);

  useEffect(() => {
    if (!selectedCategoryId) {
      setSubcategories([]);
      return;
    }

    loadSubcategories(selectedCategoryId);
  }, [selectedCategoryId]);

  async function loadCategories(nextSelectedId = selectedCategoryId) {
    setLoadingCategories(true);
    setError(null);
    try {
      const payload = await fetchJson('/api/categories');
      const rows = payload.rows || [];
      setCategories(rows);
      if (rows.length === 0) {
        setSelectedCategoryId(null);
      } else if (nextSelectedId && rows.some((row) => row.id === nextSelectedId)) {
        setSelectedCategoryId(nextSelectedId);
      } else {
        setSelectedCategoryId(rows[0].id);
      }
    } catch (loadError) {
      setError(loadError.message);
    } finally {
      setLoadingCategories(false);
    }
  }

  async function loadSubcategories(categoryId) {
    setLoadingSubcategories(true);
    setError(null);
    try {
      const payload = await fetchJson(`/api/categories/${categoryId}/subcategories`);
      setSubcategories(payload.rows || []);
    } catch (loadError) {
      setError(loadError.message);
    } finally {
      setLoadingSubcategories(false);
    }
  }

  function openCreateCategory() {
    setFormError(null);
    setModal({ kind: 'category', mode: 'create', row: null });
  }

  function openEditCategory(row) {
    setFormError(null);
    setModal({ kind: 'category', mode: 'edit', row });
  }

  function openCreateSubcategory() {
    if (!selectedCategory) {
      return;
    }

    setFormError(null);
    setModal({ kind: 'subcategory', mode: 'create', row: null });
  }

  function openEditSubcategory(row) {
    setFormError(null);
    setModal({ kind: 'subcategory', mode: 'edit', row });
  }

  async function saveModal(data) {
    if (!modal) {
      return;
    }

    setSaving(true);
    setFormError(null);
    try {
      if (modal.kind === 'category') {
        const editing = modal.mode === 'edit';
        const url = editing ? `/api/categories/${modal.row.id}` : '/api/categories';
        const payload = await fetchJson(url, {
          method: editing ? 'PUT' : 'POST',
          body: JSON.stringify(data)
        });
        setModal(null);
        await loadCategories(payload.row.id);
      } else {
        const editing = modal.mode === 'edit';
        const url = editing
          ? `/api/subcategories/${modal.row.id}`
          : `/api/categories/${selectedCategory.id}/subcategories`;
        await fetchJson(url, {
          method: editing ? 'PUT' : 'POST',
          body: JSON.stringify(data)
        });
        setModal(null);
        await loadSubcategories(selectedCategory.id);
      }
    } catch (saveError) {
      setFormError(saveError.message);
    } finally {
      setSaving(false);
    }
  }

  async function confirmDelete() {
    if (!deleteTarget) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      if (deleteTarget.kind === 'category') {
        await fetchJson(`/api/categories/${deleteTarget.row.id}`, { method: 'DELETE' });
        setDeleteTarget(null);
        await loadCategories();
      } else {
        await fetchJson(`/api/subcategories/${deleteTarget.row.id}`, { method: 'DELETE' });
        setDeleteTarget(null);
        await loadSubcategories(selectedCategory.id);
      }
    } catch (deleteError) {
      setError(deleteError.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="category-page">
      {error ? <div className="table-error">{error}</div> : null}

      <section className="split-crud-panel">
        <header className="crud-header">
          <div>
            <p className="panel-kicker">Master</p>
            <h2>Categories</h2>
          </div>
          <button className="primary-button" onClick={openCreateCategory} type="button">
            Add
          </button>
        </header>

        {loadingCategories ? (
          <div className="table-loading">Loading...</div>
        ) : (
          <DataTable
            columns={categoriesResource.columns}
            primaryKey={categoriesResource.primaryKey}
            rows={categories}
            selectedId={selectedCategoryId}
            onDelete={(row) => setDeleteTarget({ kind: 'category', row })}
            onEdit={openEditCategory}
            onSelect={(row) => setSelectedCategoryId(row.id)}
          />
        )}
      </section>

      <section className="split-crud-panel">
        <header className="crud-header">
          <div>
            <p className="panel-kicker">Detail</p>
            <h2>{selectedCategory ? `${selectedCategory.name} Subcategories` : 'Subcategories'}</h2>
          </div>
          <button
            className="primary-button"
            disabled={!selectedCategory}
            onClick={openCreateSubcategory}
            type="button"
          >
            Add
          </button>
        </header>

        {!selectedCategory ? (
          <div className="table-loading">Select a category.</div>
        ) : loadingSubcategories ? (
          <div className="table-loading">Loading...</div>
        ) : (
          <DataTable
            columns={subcategoriesResource.columns}
            primaryKey={subcategoriesResource.primaryKey}
            rows={subcategories}
            onDelete={(row) => setDeleteTarget({ kind: 'subcategory', row })}
            onEdit={openEditSubcategory}
          />
        )}
      </section>

      {modal ? (
        <CrudFormModal
          error={formError}
          mode={modal.mode}
          resource={modal.kind === 'category' ? categoriesResource : subcategoriesResource}
          row={modal.row}
          saving={saving}
          onClose={() => setModal(null)}
          onSubmit={saveModal}
        />
      ) : null}

      {deleteTarget ? (
        <ConfirmDialog
          busy={saving}
          message={`Delete "${deleteTarget.row.name}"?`}
          title={`Delete ${deleteTarget.kind === 'category' ? 'Category' : 'Subcategory'}`}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      ) : null}
    </section>
  );
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers || {})
    },
    ...options
  });

  if (response.status === 204) {
    return {};
  }

  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(payload.error || `Request failed with status ${response.status}`);
  }

  return payload;
}
