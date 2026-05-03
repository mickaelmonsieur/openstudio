const SELECT_COLS = `
  ce.id,
  ce.hour,
  ce.minute,
  ce.second,
  ce.template_id,
  ce.priority,
  ce.duration,
  t.name AS template_name
`;

const FROM_JOIN = `
  FROM clock_events ce
  LEFT JOIN templates t ON t.id = ce.template_id
`;

export async function countEvents(db) {
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total ${FROM_JOIN}`);
  return rows[0].total;
}

export async function listEvents(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${SELECT_COLS} ${FROM_JOIN} ORDER BY ce.hour, ce.minute, ce.second, ce.priority, ce.id`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} ORDER BY ce.hour, ce.minute, ce.second, ce.priority, ce.id LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return rows;
}

export async function getEvent(db, id) {
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} WHERE ce.id = $1`,
    [id]
  );
  return rows[0] || null;
}

export async function createEvent(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO clock_events (hour, minute, second, template_id, priority, duration)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING id
    `,
    [data.hour, data.minute, data.second, data.template_id, data.priority, data.duration]
  );
  return getEvent(db, rows[0].id);
}

export async function updateEvent(db, id, data) {
  await db.query(
    `
    UPDATE clock_events
    SET hour = $2,
        minute = $3,
        second = $4,
        template_id = $5,
        priority = $6,
        duration = $7
    WHERE id = $1
    `,
    [id, data.hour, data.minute, data.second, data.template_id, data.priority, data.duration]
  );
  return getEvent(db, id);
}

export async function deleteEvent(db, id) {
  const { rowCount } = await db.query('DELETE FROM clock_events WHERE id = $1', [id]);
  return rowCount > 0;
}
