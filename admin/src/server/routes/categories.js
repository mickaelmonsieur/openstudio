import { withDatabase } from '../db/client.js';
import {
  createCategory,
  createSubcategory,
  deleteCategory,
  deleteSubcategory,
  getCategory,
  getSubcategory,
  listCategories,
  listSubcategories,
  updateCategory,
  updateSubcategory
} from '../repositories/categories.js';

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function validateName(data, maxLength = 32) {
  const name = String(data?.name || '').trim();
  if (!name) {
    return { ok: false, error: 'Name is required.' };
  }

  if (name.length > maxLength) {
    return { ok: false, error: `Name must be ${maxLength} characters or less.` };
  }

  return { ok: true, value: { name } };
}

function validateSubcategory(data) {
  const base = validateName(data, 32);
  if (!base.ok) {
    return base;
  }

  return {
    ok: true,
    value: {
      ...base.value,
      hidden: Boolean(data?.hidden)
    }
  };
}

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({
        error: error.message
      });
    }
  };
}

export function registerCategoryRoutes(app, getDatabaseConfig) {
  app.get('/api/categories', asyncRoute(async (_req, res) => {
    const rows = await withDatabase(getDatabaseConfig(), listCategories);
    res.json({ rows });
  }));

  app.get('/api/categories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid category id.' });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) => getCategory(db, id));
    if (!row) {
      res.status(404).json({ error: 'Category not found.' });
      return;
    }

    res.json({ row });
  }));

  app.post('/api/categories', asyncRoute(async (req, res) => {
    const category = validateName(req.body, 32);
    if (!category.ok) {
      res.status(400).json({ error: category.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), async (db) => {
      const created = await createCategory(db, category.value);
      await createSubcategory(db, created.id, { name: 'Default', hidden: false });
      return created;
    });
    res.status(201).json({ row });
  }));

  app.put('/api/categories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid category id.' });
      return;
    }

    const existing = await withDatabase(getDatabaseConfig(), (db) => getCategory(db, id));
    if (!existing) { res.status(404).json({ error: 'Category not found.' }); return; }
    if (existing.protected) { res.status(403).json({ error: 'This category is protected.' }); return; }

    const category = validateName(req.body, 32);
    if (!category.ok) {
      res.status(400).json({ error: category.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateCategory(db, id, category.value)
    );
    res.json({ row });
  }));

  app.delete('/api/categories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid category id.' });
      return;
    }

    const existing = await withDatabase(getDatabaseConfig(), (db) => getCategory(db, id));
    if (!existing) { res.status(404).json({ error: 'Category not found.' }); return; }
    if (existing.protected) { res.status(403).json({ error: 'This category is protected.' }); return; }

    await withDatabase(getDatabaseConfig(), (db) => deleteCategory(db, id));
    res.status(204).send();
  }));

  app.get('/api/categories/:categoryId/subcategories', asyncRoute(async (req, res) => {
    const categoryId = parseId(req.params.categoryId);
    if (!categoryId) {
      res.status(400).json({ error: 'Invalid category id.' });
      return;
    }

    const rows = await withDatabase(getDatabaseConfig(), (db) =>
      listSubcategories(db, categoryId)
    );
    res.json({ rows });
  }));

  app.post('/api/categories/:categoryId/subcategories', asyncRoute(async (req, res) => {
    const categoryId = parseId(req.params.categoryId);
    if (!categoryId) {
      res.status(400).json({ error: 'Invalid category id.' });
      return;
    }

    const subcategory = validateSubcategory(req.body);
    if (!subcategory.ok) {
      res.status(400).json({ error: subcategory.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createSubcategory(db, categoryId, subcategory.value)
    );
    res.status(201).json({ row });
  }));

  app.get('/api/subcategories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid subcategory id.' });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) => getSubcategory(db, id));
    if (!row) {
      res.status(404).json({ error: 'Subcategory not found.' });
      return;
    }

    res.json({ row });
  }));

  app.put('/api/subcategories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid subcategory id.' });
      return;
    }

    const existing = await withDatabase(getDatabaseConfig(), (db) => getSubcategory(db, id));
    if (!existing) { res.status(404).json({ error: 'Subcategory not found.' }); return; }
    if (existing.protected) { res.status(403).json({ error: 'This subcategory is protected.' }); return; }

    const subcategory = validateSubcategory(req.body);
    if (!subcategory.ok) {
      res.status(400).json({ error: subcategory.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateSubcategory(db, id, subcategory.value)
    );
    res.json({ row });
  }));

  app.delete('/api/subcategories/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid subcategory id.' });
      return;
    }

    const existing = await withDatabase(getDatabaseConfig(), (db) => getSubcategory(db, id));
    if (!existing) { res.status(404).json({ error: 'Subcategory not found.' }); return; }
    if (existing.protected) { res.status(403).json({ error: 'This subcategory is protected.' }); return; }

    await withDatabase(getDatabaseConfig(), (db) => deleteSubcategory(db, id));
    res.status(204).send();
  }));
}
