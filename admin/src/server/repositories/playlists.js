const DAY_COLUMNS = {
  sunday: 'sunday',
  monday: 'monday',
  tuesday: 'tuesday',
  wednesday: 'wednesday',
  thursday: 'thursday',
  friday: 'friday',
  saturday: 'saturday'
};

export async function getConfiguredTimezone(db) {
  const { rows } = await db.query(`
    SELECT timezone
    FROM configurations
    ORDER BY id
    LIMIT 1
  `);
  return rows[0]?.timezone || 'Europe/Paris';
}

export async function currentDateInTimezone(db, timezone) {
  const { rows } = await db.query(
    "SELECT to_char((NOW() AT TIME ZONE $1)::date, 'YYYY-MM-DD') AS today",
    [timezone]
  );
  return rows[0].today;
}

export async function currentHourBoundaryInTimezone(db, timezone) {
  const { rows } = await db.query(
    "SELECT to_char(date_trunc('hour', NOW() AT TIME ZONE $1), 'YYYY-MM-DD HH24:MI:SS') AS current_hour",
    [timezone]
  );
  return rows[0].current_hour;
}

export async function countQueueInPeriod(db, fromLocal, toLocal, timezone) {
  const { rows } = await db.query(
    `
    SELECT COUNT(*)::integer AS count
    FROM queue
    WHERE scheduled_at >= ($1::timestamp AT TIME ZONE $3)
      AND scheduled_at < (($2::timestamp + INTERVAL '1 hour') AT TIME ZONE $3)
    `,
    [fromLocal, toLocal, timezone]
  );
  return rows[0].count;
}

export async function deleteQueueInPeriod(db, fromLocal, toLocal, timezone) {
  const { rowCount } = await db.query(
    `
    DELETE FROM queue
    WHERE scheduled_at >= ($1::timestamp AT TIME ZONE $3)
      AND scheduled_at < (($2::timestamp + INTERVAL '1 hour') AT TIME ZONE $3)
    `,
    [fromLocal, toLocal, timezone]
  );
  return rowCount;
}

export async function listQueueCoverage(db, timezone, days = 42) {
  const { rows } = await db.query(
    `
    SELECT
      to_char(q.scheduled_at AT TIME ZONE $1, 'YYYY-MM-DD') AS date,
      EXTRACT(HOUR FROM q.scheduled_at AT TIME ZONE $1)::integer AS hour,
      COUNT(*)::integer AS count
    FROM queue q
    WHERE q.scheduled_at >= (CURRENT_DATE::timestamp AT TIME ZONE $1)
      AND q.scheduled_at < ((CURRENT_DATE + $2::integer)::timestamp AT TIME ZONE $1)
    GROUP BY date, hour
    ORDER BY date, hour
    `,
    [timezone, days]
  );
  return rows;
}

export async function getScheduleForHour(db, dayKey, hour) {
  const column = DAY_COLUMNS[dayKey];
  if (!column) throw new Error(`Invalid schedule day: ${dayKey}`);

  const { rows } = await db.query(
    `
    SELECT s.id, s.template_id, t.name AS template_name
    FROM schedules s
    JOIN templates t ON t.id = s.template_id
    WHERE s.${column} = TRUE
      AND s.from_hour <= $1
      AND s.to_hour >= $1
    ORDER BY s.id
    LIMIT 1
    `,
    [hour]
  );
  return rows[0] || null;
}

export async function listSlotsForGenerator(db, templateId) {
  const { rows } = await db.query(
    `
    SELECT
      ts.id,
      ts.position,
      ts.category_id,
      ts.subcategory_id,
      ts.track_protection,
      ts.artist_protection,
      COALESCE(NULLIF(ts.comment, ''), c.name) AS label
    FROM template_slots ts
    JOIN categories c ON c.id = ts.category_id
    WHERE ts.template_id = $1
    ORDER BY ts.position, ts.id
    `,
    [templateId]
  );
  return rows;
}

export async function findTrackForSlot(db, slot, scheduledAtLocal, timezone) {
  const { rows } = await db.query(
    `
    WITH candidates AS (
      SELECT
        t.id,
        t.artist_id,
        t.cue_in,
        COALESCE(t.cue_out, t.duration) AS cue_out,
        CASE
          WHEN t.cue_out IS NOT NULL AND t.cue_out > t.cue_in THEN t.cue_out - t.cue_in
          ELSE GREATEST(t.duration - t.cue_in, 0)
        END AS play_duration
      FROM tracks t
      JOIN subcategories sc ON sc.id = t.subcategory_id
      LEFT JOIN artists a ON a.id = t.artist_id
      WHERE t.active = TRUE
        AND (
          ($6::integer IS NOT NULL AND t.subcategory_id = $6)
          OR ($6::integer IS NULL AND sc.category_id = $3)
        )
        AND (
          $4::integer = 0
          OR t.last_played_at IS NULL
          OR t.last_played_at <= (($1::timestamp AT TIME ZONE $2) - ($4::integer * INTERVAL '1 second'))
        )
        AND (
          $5::integer = 0
          OR t.artist_id IS NULL
          OR a.last_broadcast_at IS NULL
          OR a.last_broadcast_at <= (($1::timestamp AT TIME ZONE $2) - ($5::integer * INTERVAL '1 second'))
        )
        AND NOT EXISTS (
          SELECT 1
          FROM queue q
          WHERE q.played = FALSE
            AND q.track_id = t.id
            AND $4::integer > 0
            AND q.scheduled_at BETWEEN
              (($1::timestamp AT TIME ZONE $2) - ($4::integer * INTERVAL '1 second'))
              AND
              (($1::timestamp AT TIME ZONE $2) + ($4::integer * INTERVAL '1 second'))
        )
        AND (
          t.artist_id IS NULL
          OR NOT EXISTS (
            SELECT 1
            FROM queue q
            JOIN tracks queued_track ON queued_track.id = q.track_id
            WHERE q.played = FALSE
              AND queued_track.artist_id = t.artist_id
              AND $5::integer > 0
              AND q.scheduled_at BETWEEN
                (($1::timestamp AT TIME ZONE $2) - ($5::integer * INTERVAL '1 second'))
                AND
                (($1::timestamp AT TIME ZONE $2) + ($5::integer * INTERVAL '1 second'))
          )
        )
    )
    SELECT *
    FROM candidates
    WHERE play_duration > 0
    ORDER BY random()
    LIMIT 1
    `,
    [
      scheduledAtLocal,
      timezone,
      slot.category_id,
      slot.track_protection,
      slot.artist_protection,
      slot.subcategory_id
    ]
  );
  return rows[0] || null;
}

export async function insertQueueEntry(db, track, scheduledAtLocal, timezone, priority) {
  await db.query(
    `
    INSERT INTO queue (track_id, cue_in, cue_out, priority, fixed_time, scheduled_at)
    VALUES ($1, $2, $3, $4, FALSE, $5::timestamp AT TIME ZONE $6)
    `,
    [track.id, track.cue_in, track.cue_out, priority, scheduledAtLocal, timezone]
  );
}
