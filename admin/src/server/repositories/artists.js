export async function listArtists(db) {
  const { rows } = await db.query(`
    SELECT
      id,
      name,
      to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
    FROM artists
    ORDER BY name
  `);

  return rows;
}

export async function getArtist(db, id) {
  const { rows } = await db.query(
    `
    SELECT
      id,
      name,
      to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
    FROM artists
    WHERE id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createArtist(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO artists (name)
    VALUES ($1)
    RETURNING
      id,
      name,
      to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
    `,
    [data.name.trim()]
  );

  return rows[0];
}

export async function findOrCreateArtist(db, name) {
  const normalizedName = String(name || '').trim();
  if (!normalizedName) return null;

  const existing = await db.query(
    `
    SELECT
      id,
      name,
      to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
    FROM artists
    WHERE lower(name) = lower($1)
    LIMIT 1
    `,
    [normalizedName]
  );

  if (existing.rows[0]) return existing.rows[0];

  try {
    return await createArtist(db, { name: normalizedName });
  } catch (error) {
    if (error.code !== '23505') throw error;

    const retry = await db.query(
      `
      SELECT
        id,
        name,
        to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
      FROM artists
      WHERE name = $1
      LIMIT 1
      `,
      [normalizedName]
    );

    return retry.rows[0] || null;
  }
}

export async function updateArtist(db, id, data) {
  const { rows } = await db.query(
    `
    UPDATE artists
    SET name = $2
    WHERE id = $1
    RETURNING
      id,
      name,
      to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at
    `,
    [id, data.name.trim()]
  );

  return rows[0] || null;
}

export async function deleteArtist(db, id) {
  const { rowCount } = await db.query(
    `
    DELETE FROM artists
    WHERE id = $1
    `,
    [id]
  );

  return rowCount > 0;
}
