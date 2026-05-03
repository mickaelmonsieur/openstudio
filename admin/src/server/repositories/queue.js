const HOUR_LIMIT_SECONDS = 3599.999;

export async function getQueueTimezone(db) {
  const { rows } = await db.query(`
    SELECT timezone
    FROM configurations
    ORDER BY id
    LIMIT 1
  `);
  return rows[0]?.timezone || 'Europe/Paris';
}

export async function listQueueHour(db, { date, hour, timezone }) {
  const { rows } = await db.query(
    `
    SELECT
      q.id,
      q.track_id,
      q.cue_in,
      q.cue_out,
      q.stretch_rate,
      q.played,
      q.priority,
      q.fixed_time,
      to_char(q.scheduled_at AT TIME ZONE $2, 'YYYY-MM-DD') AS scheduled_date,
      to_char(q.scheduled_at AT TIME ZONE $2, 'HH24:MI:SS') AS scheduled_time,
      COALESCE(a.name, '') AS artist,
      COALESCE(t.title, '') AS title,
      COALESCE(t.duration, 0)::double precision AS duration
    FROM queue q
    LEFT JOIN tracks t ON t.id = q.track_id
    LEFT JOIN artists a ON a.id = t.artist_id
    WHERE q.scheduled_at >= ($1::timestamp AT TIME ZONE $2)
      AND q.scheduled_at < (($1::timestamp + INTERVAL '1 hour') AT TIME ZONE $2)
    ORDER BY q.scheduled_at, q.priority, q.id
    `,
    [hourBoundary(date, hour), timezone]
  );
  return rows;
}

export async function createQueueEntryInHour(db, data, timezone) {
  await db.query('BEGIN');
  try {
    const { rows } = await db.query(
      `
      INSERT INTO queue (track_id, cue_in, cue_out, stretch_rate, played, priority, fixed_time, scheduled_at)
      VALUES ($1, $2, $3, $4, FALSE, 0, FALSE, $5::timestamp AT TIME ZONE $6)
      RETURNING id
      `,
      [
        data.track_id,
        data.cue_in,
        data.cue_out,
        data.stretch_rate,
        hourBoundary(data.scheduled_date, data.scheduled_hour),
        timezone
      ]
    );

    const insertedId = rows[0].id;
    const ids = await orderedIdsForHour(db, data.scheduled_date, data.scheduled_hour, timezone);
    const nextIds = insertAfter(ids.filter((id) => id !== insertedId), insertedId, data.insert_after_id);
    await recalculateQueueHour(db, data.scheduled_date, data.scheduled_hour, nextIds, timezone);
    await db.query('COMMIT');
    return listQueueHour(db, { date: data.scheduled_date, hour: data.scheduled_hour, timezone });
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

export async function updateQueueEntryInHour(db, id, data, timezone) {
  await db.query('BEGIN');
  try {
    const { rowCount } = await db.query(
      `
      UPDATE queue
      SET track_id = $2,
          cue_in = $3,
          cue_out = $4,
          stretch_rate = $5,
          updated_at = NOW()
      WHERE id = $1
      `,
      [id, data.track_id, data.cue_in, data.cue_out, data.stretch_rate]
    );
    if (rowCount === 0) {
      await db.query('ROLLBACK');
      return null;
    }

    const ids = await orderedIdsForHour(db, data.scheduled_date, data.scheduled_hour, timezone);
    await recalculateQueueHour(db, data.scheduled_date, data.scheduled_hour, ids, timezone);
    await db.query('COMMIT');
    return listQueueHour(db, { date: data.scheduled_date, hour: data.scheduled_hour, timezone });
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

export async function deleteQueueEntryFromHour(db, id, date, hour, timezone) {
  await db.query('BEGIN');
  try {
    const { rowCount } = await db.query('DELETE FROM queue WHERE id = $1', [id]);
    if (rowCount === 0) {
      await db.query('ROLLBACK');
      return false;
    }

    const ids = await orderedIdsForHour(db, date, hour, timezone);
    await recalculateQueueHour(db, date, hour, ids, timezone);
    await db.query('COMMIT');
    return true;
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

export async function reorderQueueHour(db, date, hour, ids, timezone) {
  await db.query('BEGIN');
  try {
    const existingIds = await orderedIdsForHour(db, date, hour, timezone);
    const same = existingIds.length === ids.length
      && new Set(ids).size === ids.length
      && ids.every((id) => existingIds.includes(id));
    if (!same) {
      const error = new Error('Queue order does not match this hour.');
      error.statusCode = 400;
      throw error;
    }

    await recalculateQueueHour(db, date, hour, ids, timezone);
    await db.query('COMMIT');
    return listQueueHour(db, { date, hour, timezone });
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

async function orderedIdsForHour(db, date, hour, timezone) {
  const rows = await listQueueHour(db, { date, hour, timezone });
  return rows.map((row) => row.id);
}

async function recalculateQueueHour(db, date, hour, ids, timezone) {
  let offsetSeconds = 0;
  const start = hourBoundary(date, hour);

  for (let index = 0; index < ids.length; index += 1) {
    const { rows } = await db.query(
      'SELECT cue_in, cue_out, stretch_rate FROM queue WHERE id = $1',
      [ids[index]]
    );
    if (!rows[0]) continue;

    const cueIn = Number(rows[0].cue_in || 0);
    const cueOut = Number(rows[0].cue_out || 0);
    const stretchRate = Number(rows[0].stretch_rate || 1);
    const playDuration = Math.max(0, (cueOut - cueIn) / stretchRate);

    if (offsetSeconds + playDuration > HOUR_LIMIT_SECONDS) {
      throw new Error('This order exceeds the selected hour.');
    }

    await db.query(
      `
      UPDATE queue
      SET scheduled_at = (($2::timestamp + ($3::double precision * INTERVAL '1 second')) AT TIME ZONE $4),
          priority = $5,
          updated_at = NOW()
      WHERE id = $1
      `,
      [ids[index], start, offsetSeconds, timezone, index + 1]
    );

    offsetSeconds += playDuration;
  }
}

function insertAfter(ids, insertedId, insertAfterId) {
  if (!insertAfterId) return [...ids, insertedId];

  const index = ids.indexOf(insertAfterId);
  if (index < 0) return [...ids, insertedId];

  const next = [...ids];
  next.splice(index + 1, 0, insertedId);
  return next;
}

function hourBoundary(date, hour) {
  return `${date} ${String(hour).padStart(2, '0')}:00:00`;
}
