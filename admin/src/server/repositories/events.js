const SELECT_COLS = `
  ce.id,
  ce.event_type,
  ce.days_mask,
  ce.hours_mask,
  TO_CHAR(ce.event_date, 'YYYY-MM-DD') AS event_date,
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

const ORDER = `ORDER BY ce.event_type, ce.event_date, ce.days_mask, ce.hour, ce.minute, ce.second, ce.priority, ce.id`;

export async function countEvents(db) {
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total ${FROM_JOIN}`);
  return rows[0].total;
}

export async function listEvents(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${SELECT_COLS} ${FROM_JOIN} ${ORDER}`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} ${ORDER} LIMIT $1 OFFSET $2`,
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
    INSERT INTO clock_events
      (event_type, days_mask, hours_mask, event_date, hour, minute, second, template_id, priority, duration)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    RETURNING id
    `,
    [data.event_type, data.days_mask, data.hours_mask, data.event_date,
     data.hour, data.minute, data.second, data.template_id, data.priority, data.duration]
  );
  return getEvent(db, rows[0].id);
}

export async function updateEvent(db, id, data) {
  await db.query(
    `
    UPDATE clock_events
    SET event_type  = $2,
        days_mask   = $3,
        hours_mask  = $4,
        event_date  = $5,
        hour        = $6,
        minute      = $7,
        second      = $8,
        template_id = $9,
        priority    = $10,
        duration    = $11
    WHERE id = $1
    `,
    [id, data.event_type, data.days_mask, data.hours_mask, data.event_date,
     data.hour, data.minute, data.second, data.template_id, data.priority, data.duration]
  );
  return getEvent(db, id);
}

export async function deleteEvent(db, id) {
  const { rowCount } = await db.query('DELETE FROM clock_events WHERE id = $1', [id]);
  return rowCount > 0;
}
