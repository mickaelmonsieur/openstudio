export async function listStations(db) {
  const { rows } = await db.query(`
    SELECT id, name
    FROM stations
    ORDER BY name
  `);

  return rows;
}

export async function getStation(db, id) {
  const { rows } = await db.query(
    `
    SELECT id, name
    FROM stations
    WHERE id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createStation(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO stations (name)
    VALUES ($1)
    RETURNING id, name
    `,
    [data.name.trim()]
  );

  return rows[0];
}

export async function updateStation(db, id, data) {
  const { rows } = await db.query(
    `
    UPDATE stations
    SET name = $2
    WHERE id = $1
    RETURNING id, name
    `,
    [id, data.name.trim()]
  );

  return rows[0] || null;
}

export async function deleteStation(db, id) {
  const { rowCount } = await db.query(
    `
    DELETE FROM stations
    WHERE id = $1
    `,
    [id]
  );

  return rowCount > 0;
}
