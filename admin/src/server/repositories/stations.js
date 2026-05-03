const COLUMNS = 'id, name, library_path';

export async function countStations(db) {
  const { rows } = await db.query('SELECT COUNT(*)::integer AS total FROM stations');
  return rows[0].total;
}

export async function listStations(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${COLUMNS} FROM stations ORDER BY name`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${COLUMNS} FROM stations ORDER BY name LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return rows;
}

export async function getStation(db, id) {
  const { rows } = await db.query(
    `SELECT ${COLUMNS} FROM stations WHERE id = $1`,
    [id]
  );

  return rows[0] || null;
}

export async function createStation(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO stations (name, library_path)
    VALUES ($1, $2)
    RETURNING ${COLUMNS}
    `,
    [data.name.trim(), data.library_path.trim()]
  );

  return rows[0];
}

export async function updateStation(db, id, data) {
  const { rows } = await db.query(
    `
    UPDATE stations
    SET name         = $2,
        library_path = $3
    WHERE id = $1
    RETURNING ${COLUMNS}
    `,
    [id, data.name.trim(), data.library_path.trim()]
  );

  return rows[0] || null;
}

export async function deleteStation(db, id) {
  const { rowCount } = await db.query(
    'DELETE FROM stations WHERE id = $1',
    [id]
  );

  return rowCount > 0;
}
