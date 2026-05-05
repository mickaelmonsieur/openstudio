const SELECT_COLS = `
  ce.id,
  ce.event_type,
  ce.days_mask,
  ce.hours_mask,
  TO_CHAR(ce.event_date, 'YYYY-MM-DD') AS event_date,
  ce.hour,
  ce.minute,
  ce.second,
  ce.priority,
  ce.duration,
  COALESCE(
    json_agg(
      json_build_object(
        'id',            ea.id,
        'action_type',   ea.action_type,
        'template_id',   ea.template_id,
        'template_name', t.name,
        'track_id',      ea.track_id,
        'track_name',    COALESCE(a.name || ' — ', '') || tr.title
      )
      ORDER BY ea.id
    ) FILTER (WHERE ea.id IS NOT NULL),
    '[]'::json
  ) AS actions
`;

const FROM_JOIN = `
  FROM clock_events ce
  LEFT JOIN event_actions ea  ON ea.event_id  = ce.id
  LEFT JOIN templates t       ON t.id          = ea.template_id
  LEFT JOIN tracks tr         ON tr.id         = ea.track_id
  LEFT JOIN artists a         ON a.id          = tr.artist_id
`;

const GROUP_BY = `GROUP BY ce.id`;

const ORDER = `ORDER BY ce.event_type, ce.event_date, ce.days_mask, ce.hour, ce.minute, ce.second, ce.priority, ce.id`;

export async function countEvents(db) {
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total FROM clock_events`);
  return rows[0].total;
}

export async function listEvents(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${SELECT_COLS} ${FROM_JOIN} ${GROUP_BY} ${ORDER}`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} ${GROUP_BY} ${ORDER} LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return rows;
}

export async function getEvent(db, id) {
  const { rows } = await db.query(
    `SELECT ${SELECT_COLS} ${FROM_JOIN} WHERE ce.id = $1 ${GROUP_BY}`,
    [id]
  );
  return rows[0] || null;
}

export async function createEvent(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO clock_events
      (event_type, days_mask, hours_mask, event_date, hour, minute, second, priority, duration)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    RETURNING id
    `,
    [data.event_type, data.days_mask, data.hours_mask, data.event_date,
     data.hour, data.minute, data.second, data.priority, data.duration]
  );
  const eventId = rows[0].id;
  await insertActions(db, eventId, data.actions);
  return getEvent(db, eventId);
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
        priority    = $9,
        duration    = $10
    WHERE id = $1
    `,
    [id, data.event_type, data.days_mask, data.hours_mask, data.event_date,
     data.hour, data.minute, data.second, data.priority, data.duration]
  );
  await db.query(`DELETE FROM event_actions WHERE event_id = $1`, [id]);
  await insertActions(db, id, data.actions);
  return getEvent(db, id);
}

export async function deleteEvent(db, id) {
  const { rowCount } = await db.query('DELETE FROM clock_events WHERE id = $1', [id]);
  return rowCount > 0;
}

async function insertActions(db, eventId, actions) {
  for (const action of actions) {
    await db.query(
      `INSERT INTO event_actions (event_id, action_type, template_id, track_id)
       VALUES ($1, $2, $3, $4)`,
      [eventId, action.action_type, action.template_id || null, action.track_id || null]
    );
  }
}
