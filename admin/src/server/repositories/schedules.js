const SELECT_COLS = `
  s.id,
  s.from_hour,
  s.to_hour,
  s.monday,
  s.tuesday,
  s.wednesday,
  s.thursday,
  s.friday,
  s.saturday,
  s.sunday,
  s.template_id,
  t.name AS template_name
`;

const FROM_JOIN = `
  FROM schedules s
  LEFT JOIN templates t ON t.id = s.template_id
`;

export async function listSchedules(db) {
  const { rows } = await db.query(`
    SELECT ${SELECT_COLS}
    ${FROM_JOIN}
    ORDER BY s.from_hour, s.to_hour, s.id
  `);
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
    INSERT INTO schedules
      (from_hour, to_hour, monday, tuesday, wednesday, thursday, friday, saturday, sunday, template_id)
    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
    RETURNING id
    `,
    [
      data.from_hour, data.to_hour,
      data.monday, data.tuesday, data.wednesday,
      data.thursday, data.friday, data.saturday, data.sunday,
      data.template_id
    ]
  );
  return getSchedule(db, rows[0].id);
}

export async function updateSchedule(db, id, data) {
  await db.query(
    `
    UPDATE schedules
    SET from_hour   = $2,
        to_hour     = $3,
        monday      = $4,
        tuesday     = $5,
        wednesday   = $6,
        thursday    = $7,
        friday      = $8,
        saturday    = $9,
        sunday      = $10,
        template_id = $11
    WHERE id = $1
    `,
    [
      id,
      data.from_hour, data.to_hour,
      data.monday, data.tuesday, data.wednesday,
      data.thursday, data.friday, data.saturday, data.sunday,
      data.template_id
    ]
  );
  return getSchedule(db, id);
}

export async function deleteSchedule(db, id) {
  const { rowCount } = await db.query('DELETE FROM schedules WHERE id = $1', [id]);
  return rowCount > 0;
}
