export async function countArtists(db, search = '') {
  const { where, values } = buildSearchWhere(search);
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total FROM artists ${where}`, values);
  return rows[0].total;
}

export async function listArtists(db, search = '', { limit, offset } = {}) {
  const { where, values } = buildSearchWhere(search);
  if (limit == null) {
    const { rows } = await db.query(
      `SELECT id, name, to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at FROM artists ${where} ORDER BY name`,
      values
    );
    return rows;
  }
  const { rows } = await db.query(
    `SELECT id, name, to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS') AS last_broadcast_at FROM artists ${where} ORDER BY name LIMIT $${values.length + 1} OFFSET $${values.length + 2}`,
    [...values, limit, offset]
  );
  return rows;
}

function buildSearchWhere(search) {
  const terms = String(search || '')
    .trim()
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 8);

  if (terms.length === 0) {
    return { where: '', values: [] };
  }

  const values = terms.map((term) => `%${escapeLike(term)}%`);
  const clauses = values.map((_, index) => {
    const param = `$${index + 1}`;
    return `
      (
        id::text ILIKE ${param} ESCAPE '\\'
        OR name ILIKE ${param} ESCAPE '\\'
        OR COALESCE(to_char(last_broadcast_at, 'YYYY-MM-DD HH24:MI:SS'), '') ILIKE ${param} ESCAPE '\\'
      )
    `;
  });

  return {
    where: `WHERE ${clauses.join(' AND ')}`,
    values
  };
}

function escapeLike(value) {
  return String(value).replace(/[\\%_]/g, '\\$&');
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
