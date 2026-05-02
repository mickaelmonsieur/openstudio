export async function countAutomixLog(db) {
  const { rows } = await db.query('SELECT COUNT(*)::integer AS total FROM automix_log');
  return rows[0].total;
}

export async function listAutomixLog(db, { limit, offset }) {
  const { rows } = await db.query(
    `
    SELECT
      id,
      to_char(logged_at, 'YYYY-MM-DD HH24:MI:SS') AS logged_at,
      message
    FROM automix_log
    ORDER BY logged_at DESC
    LIMIT $1 OFFSET $2
    `,
    [limit, offset]
  );

  return rows;
}
