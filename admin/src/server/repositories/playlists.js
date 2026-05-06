// bit 0=Mon, 1=Tue, 2=Wed, 3=Thu, 4=Fri, 5=Sat, 6=Sun
const DAY_BITS = {
  monday: 0, tuesday: 1, wednesday: 2, thursday: 3,
  friday: 4, saturday: 5, sunday: 6
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

export async function copyQueuePeriod(db, fromLocal, toLocal, destLocal, timezone) {
  const { rowCount } = await db.query(
    `
    INSERT INTO queue (track_id, cue_in, cue_out, stretch_rate, played, priority, fixed_time, scheduled_at)
    SELECT
      track_id,
      cue_in,
      cue_out,
      stretch_rate,
      FALSE,
      priority,
      fixed_time,
      scheduled_at + (($3::timestamp AT TIME ZONE $4) - ($1::timestamp AT TIME ZONE $4))
    FROM queue
    WHERE scheduled_at >= ($1::timestamp AT TIME ZONE $4)
      AND scheduled_at < (($2::timestamp + INTERVAL '1 hour') AT TIME ZONE $4)
    `,
    [fromLocal, toLocal, destLocal, timezone]
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
  const bit = DAY_BITS[dayKey];
  if (bit === undefined) throw new Error(`Invalid schedule day: ${dayKey}`);

  const { rows } = await db.query(
    `
    SELECT s.id, s.template_id, t.name AS template_name
    FROM schedules s
    JOIN templates t ON t.id = s.template_id
    WHERE (s.days_mask & $2) > 0
      AND s.from_hour <= $1
      AND s.to_hour >= $1
    ORDER BY s.id
    LIMIT 1
    `,
    [hour, 1 << bit]
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
        t.title,
        t.artist_id,
        a.name AS artist_name,
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

export async function insertQueueEntryWithCutoff(db, track, maxDuration, scheduledAtLocal, timezone, priority) {
  const cutCueOut = track.cue_in + maxDuration;
  const actualCueOut = Math.min(track.cue_out, cutCueOut);
  await db.query(
    `
    INSERT INTO queue (track_id, cue_in, cue_out, priority, fixed_time, scheduled_at)
    VALUES ($1, $2, $3, $4, FALSE, $5::timestamp AT TIME ZONE $6)
    `,
    [track.id, track.cue_in, actualCueOut, priority, scheduledAtLocal, timezone]
  );
}

export async function insertFixedQueueEntry(db, track, priority, scheduledAtLocal, timezone) {
  await db.query(
    `
    INSERT INTO queue (track_id, cue_in, cue_out, priority, fixed_time, scheduled_at)
    VALUES ($1, $2, $3, $4, TRUE, $5::timestamp AT TIME ZONE $6)
    `,
    [track.id, track.cue_in, track.cue_out, priority, scheduledAtLocal, timezone]
  );
}

export async function getTrackById(db, id) {
  const { rows } = await db.query(
    `
    SELECT t.id, t.title, t.cue_in, COALESCE(t.cue_out, t.duration) AS cue_out,
      a.name AS artist_name
    FROM tracks t
    LEFT JOIN artists a ON a.id = t.artist_id
    WHERE t.id = $1 AND t.active = TRUE
    `,
    [id]
  );
  return rows[0] || null;
}

export async function listEventsForHour(db, date, dayKey, hour) {
  const bit = DAY_BITS[dayKey];
  if (bit === undefined) throw new Error(`Invalid schedule day: ${dayKey}`);
  const dayBit  = 1 << bit;
  const hourBit = 1 << hour;

  const { rows } = await db.query(
    `
    SELECT
      ce.id          AS event_id,
      ce.name,
      ce.event_type,
      ce.is_fixed,
      ce.minute,
      ce.second,
      ce.priority,
      ce.duration,
      ea.id          AS action_id,
      ea.action_type,
      ea.template_id,
      ea.track_id
    FROM clock_events ce
    JOIN event_actions ea ON ea.event_id = ce.id
    WHERE (
      (ce.event_type = 1 AND ce.event_date = $1::date AND ce.hour = $3)
      OR (ce.event_type = 2 AND (ce.days_mask & $2) > 0 AND ce.hour = $3)
      OR (ce.event_type = 3 AND (ce.days_mask & $2) > 0 AND (ce.hours_mask & $4) > 0)
      OR (ce.event_type = 4
          AND TO_CHAR(ce.event_date, 'MM-DD') = TO_CHAR($1::date, 'MM-DD')
          AND ce.hour = $3)
      OR (ce.event_type = 5
          AND TO_CHAR(ce.event_date, 'MM-DD') = TO_CHAR($1::date, 'MM-DD')
          AND (ce.hours_mask & $4) > 0)
    )
    ORDER BY ce.minute, ce.second, ce.priority DESC, ce.id, ea.id
    `,
    [date, dayBit, hour, hourBit]
  );
  return rows;
}
