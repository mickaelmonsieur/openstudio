const USER_COLUMNS = `
  u.id,
  u.login,
  u.active,
  u.role_id,
  r.name AS role_name
`;

const FROM_JOIN = `
  FROM users u
  JOIN users_roles r ON r.id = u.role_id
`;

export async function countUsers(db) {
  const { rows } = await db.query(`SELECT COUNT(*)::integer AS total ${FROM_JOIN}`);
  return rows[0].total;
}

export async function listUsers(db, { limit, offset } = {}) {
  if (limit == null) {
    const { rows } = await db.query(`SELECT ${USER_COLUMNS} ${FROM_JOIN} ORDER BY u.login`);
    return rows;
  }
  const { rows } = await db.query(
    `SELECT ${USER_COLUMNS} ${FROM_JOIN} ORDER BY u.login LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return rows;
}

export async function getUser(db, id) {
  const { rows } = await db.query(
    `
    SELECT ${USER_COLUMNS}
    ${FROM_JOIN}
    WHERE u.id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createUser(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO users (login, password_hash, active, role_id)
    VALUES ($1, crypt($2, gen_salt('bf')), $3, $4)
    RETURNING id
    `,
    [data.login, data.password, data.active, data.role_id]
  );

  return getUser(db, rows[0].id);
}

export async function updateUser(db, id, data) {
  if (data.password) {
    await db.query(
      `
      UPDATE users
      SET login         = $2,
          password_hash = crypt($3, gen_salt('bf')),
          active        = $4,
          role_id       = $5
      WHERE id = $1
      `,
      [id, data.login, data.password, data.active, data.role_id]
    );
  } else {
    await db.query(
      `
      UPDATE users
      SET login   = $2,
          active  = $3,
          role_id = $4
      WHERE id = $1
      `,
      [id, data.login, data.active, data.role_id]
    );
  }

  return getUser(db, id);
}

export async function deleteUser(db, id) {
  const { rowCount } = await db.query(
    `
    DELETE FROM users
    WHERE id = $1
    `,
    [id]
  );

  return rowCount > 0;
}
