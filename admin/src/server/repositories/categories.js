export async function listCategories(db) {
  const { rows } = await db.query(`
    SELECT id, name, protected
    FROM categories
    ORDER BY name
  `);

  return rows;
}

export async function getCategory(db, id) {
  const { rows } = await db.query(
    `
    SELECT id, name, protected
    FROM categories
    WHERE id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createCategory(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO categories (name)
    VALUES ($1)
    RETURNING id, name
    `,
    [data.name.trim()]
  );

  return rows[0];
}

export async function updateCategory(db, id, data) {
  const { rows } = await db.query(
    `
    UPDATE categories
    SET name = $2
    WHERE id = $1
    RETURNING id, name
    `,
    [id, data.name.trim()]
  );

  return rows[0] || null;
}

export async function deleteCategory(db, id) {
  const { rowCount } = await db.query(
    `
    DELETE FROM categories
    WHERE id = $1
    `,
    [id]
  );

  return rowCount > 0;
}

export async function listSubcategories(db, categoryId) {
  const { rows } = await db.query(
    `
    SELECT id, category_id, name, hidden, protected
    FROM subcategories
    WHERE category_id = $1
    ORDER BY hidden, name
    `,
    [categoryId]
  );

  return rows;
}

export async function getSubcategory(db, id) {
  const { rows } = await db.query(
    `
    SELECT id, category_id, name, hidden, protected
    FROM subcategories
    WHERE id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createSubcategory(db, categoryId, data) {
  const { rows } = await db.query(
    `
    INSERT INTO subcategories (category_id, name, hidden)
    VALUES ($1, $2, $3)
    RETURNING id, category_id, name, hidden
    `,
    [categoryId, data.name.trim(), Boolean(data.hidden)]
  );

  return rows[0];
}

export async function updateSubcategory(db, id, data) {
  const { rows } = await db.query(
    `
    UPDATE subcategories
    SET name = $2,
        hidden = $3
    WHERE id = $1
    RETURNING id, category_id, name, hidden
    `,
    [id, data.name.trim(), Boolean(data.hidden)]
  );

  return rows[0] || null;
}

export async function deleteSubcategory(db, id) {
  const { rowCount } = await db.query(
    `
    DELETE FROM subcategories
    WHERE id = $1
    `,
    [id]
  );

  return rowCount > 0;
}
