const SLOT_SELECT = `
  ts.id,
  ts.template_id,
  ts.position,
  ts.category_id,
  ts.subcategory_id,
  ts.comment,
  ts.track_protection,
  ts.artist_protection,
  c.name  AS category_name,
  sc.name AS subcategory_name
`;

const SLOT_FROM = `
  FROM template_slots ts
  LEFT JOIN categories c     ON c.id  = ts.category_id
  LEFT JOIN subcategories sc ON sc.id = ts.subcategory_id
`;

export async function listTemplateSlots(db) {
  const { rows } = await db.query(`
    SELECT
      ts.id,
      COALESCE(NULLIF(ts.comment, ''), 'Slot ' || ts.id::text) AS name,
      ts.comment,
      ts.template_id,
      ts.position,
      ts.category_id,
      ts.subcategory_id,
      ts.track_protection,
      ts.artist_protection,
      t.name AS template_name,
      c.name AS category_name
    FROM template_slots ts
    LEFT JOIN templates t  ON t.id = ts.template_id
    LEFT JOIN categories c ON c.id = ts.category_id
    ORDER BY t.name, ts.position, ts.id
  `);
  return rows;
}

export async function listSlotsForTemplate(db, templateId) {
  const { rows } = await db.query(
    `SELECT ${SLOT_SELECT} ${SLOT_FROM} WHERE ts.template_id = $1 ORDER BY ts.position, ts.id`,
    [templateId]
  );
  return rows;
}

export async function getTemplateSlot(db, id) {
  const { rows } = await db.query(
    `SELECT ${SLOT_SELECT} ${SLOT_FROM} WHERE ts.id = $1`,
    [id]
  );
  return rows[0] || null;
}

export async function createTemplateSlot(db, templateId, data) {
  await db.query('BEGIN');
  try {
    const position = await getInsertPosition(db, templateId, data.insert_after_id);

    await db.query(
      'UPDATE template_slots SET position = position + 1 WHERE template_id = $1 AND position >= $2',
      [templateId, position]
    );

    const { rows } = await db.query(
      `INSERT INTO template_slots (template_id, position, category_id, subcategory_id, comment, track_protection, artist_protection)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       RETURNING id`,
      [
        templateId,
        position,
        data.category_id,
        data.subcategory_id || null,
        data.comment,
        data.track_protection,
        data.artist_protection
      ]
    );

    const row = await getTemplateSlot(db, rows[0].id);
    await db.query('COMMIT');
    return row;
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

async function getInsertPosition(db, templateId, insertAfterId) {
  if (!insertAfterId) {
    const { rows } = await db.query(
      'SELECT COALESCE(MAX(position) + 1, 1) AS position FROM template_slots WHERE template_id = $1',
      [templateId]
    );
    return rows[0].position;
  }

  const { rows } = await db.query(
    'SELECT position FROM template_slots WHERE template_id = $1 AND id = $2',
    [templateId, insertAfterId]
  );

  if (!rows[0]) {
    const error = new Error('Insert position does not match this template.');
    error.statusCode = 400;
    throw error;
  }

  return rows[0].position + 1;
}

export async function updateTemplateSlot(db, id, data) {
  await db.query(
    `UPDATE template_slots
     SET category_id = $2, subcategory_id = $3, comment = $4, track_protection = $5, artist_protection = $6
     WHERE id = $1`,
    [id, data.category_id, data.subcategory_id || null, data.comment, data.track_protection, data.artist_protection]
  );
  return getTemplateSlot(db, id);
}

export async function deleteTemplateSlot(db, id) {
  const { rowCount } = await db.query('DELETE FROM template_slots WHERE id = $1', [id]);
  return rowCount > 0;
}

export async function reorderTemplateSlots(db, templateId, ids) {
  await db.query('BEGIN');
  try {
    const { rows } = await db.query(
      'SELECT id FROM template_slots WHERE template_id = $1 ORDER BY position, id',
      [templateId]
    );
    const existingIds = rows.map((row) => row.id);
    const expected = new Set(existingIds);
    const received = new Set(ids);
    const sameSlots = existingIds.length === ids.length
      && received.size === ids.length
      && ids.every((id) => expected.has(id));

    if (!sameSlots) {
      const error = new Error('Slot order does not match this template.');
      error.statusCode = 400;
      throw error;
    }

    for (let index = 0; index < ids.length; index += 1) {
      await db.query(
        'UPDATE template_slots SET position = $3 WHERE template_id = $1 AND id = $2',
        [templateId, ids[index], index + 1]
      );
    }

    await db.query('COMMIT');
    return listSlotsForTemplate(db, templateId);
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

export async function getSlotOptions(db) {
  const [catRows, subcatRows] = await Promise.all([
    db.query('SELECT id, name FROM categories ORDER BY name'),
    db.query('SELECT id, category_id, name FROM subcategories ORDER BY category_id, name')
  ]);
  return { categories: catRows.rows, subcategories: subcatRows.rows };
}
