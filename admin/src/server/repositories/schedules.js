const SELECT_COLS = `
  s.id,
  s.from_hour,
  s.to_hour,
  s.days_mask,
  s.template_id,
  t.name AS template_name
`;

const FROM_JOIN = `
  FROM schedules s
  LEFT JOIN templates t ON t.id = s.template_id
`;

export async function countSchedules(db) {
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total ${FROM_JOIN}`);
  return rows[0].total;
}

export async function listSchedules(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${SELECT_COLS} ${FROM_JOIN} ORDER BY s.from_hour, s.to_hour, s.id`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} ORDER BY s.from_hour, s.to_hour, s.id LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return rows;
}

export async function getSchedule(db, id) {
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} WHERE s.id = $1`,
    [id]
  );
  return rows[0] || null;
}

export async function createSchedule(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO schedules (from_hour, to_hour, days_mask, template_id)
    VALUES ($1, $2, $3, $4)
    RETURNING id
    `,
    [data.from_hour, data.to_hour, data.days_mask, data.template_id]
  );
  return getSchedule(db, rows[0].id);
}

export async function updateSchedule(db, id, data) {
  await db.query(
    `
    UPDATE schedules
    SET from_hour   = $2,
        to_hour     = $3,
        days_mask   = $4,
        template_id = $5
    WHERE id = $1
    `,
    [id, data.from_hour, data.to_hour, data.days_mask, data.template_id]
  );
  return getSchedule(db, id);
}

export async function deleteSchedule(db, id) {
  const { rowCount } = await db.query('DELETE FROM schedules WHERE id = $1', [id]);
  return rowCount > 0;
}
