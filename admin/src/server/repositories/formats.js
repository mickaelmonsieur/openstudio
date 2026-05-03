export async function listTemplates(db) {
  const { rows } = await db.query(`
    SELECT id, name
    FROM templates
    ORDER BY name
  `);
  return rows;
}

export async function getTemplate(db, id) {
  const { rows } = await db.query(
    'SELECT id, name FROM templates WHERE id = $1',
    [id]
  );
  return rows[0] || null;
}

export async function createTemplate(db, data) {
  const { rows } = await db.query(
    'INSERT INTO templates (name) VALUES ($1) RETURNING id, name',
    [data.name.trim()]
  );
  return rows[0];
}

export async function updateTemplate(db, id, data) {
  const { rows } = await db.query(
    'UPDATE templates SET name = $2 WHERE id = $1 RETURNING id, name',
    [id, data.name.trim()]
  );
  return rows[0] || null;
}

export async function deleteTemplate(db, id) {
  const { rowCount } = await db.query(
    'DELETE FROM templates WHERE id = $1',
    [id]
  );
  return rowCount > 0;
}
