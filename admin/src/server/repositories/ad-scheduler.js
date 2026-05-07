const FILLER_TRACK_TYPE_ID = 14;
const EPSILON = 0.001;

export async function listAdBreakCoverage(db, timezone, days = 7) {
  const rows = await listAdSchedulerRows(db, timezone, {
    fromLocal: null,
    toLocal: null,
    days
  });
  return summarizeBreaks(groupAdBreaks(rows));
}

export async function generateAdSchedule(db, options) {
  const timezone = options.timezone;
  const stationId = options.stationId || null;
  const fromLocal = hourBoundary(options.fromDate, options.fromHour);
  const toLocal = hourBoundary(options.toDate, options.toHour);

  await db.query('BEGIN');
  try {
    const rows = await listAdSchedulerRows(db, timezone, { fromLocal, toLocal });
    const breaks = groupAdBreaks(rows).filter((adBreak) =>
      adBreak.fillerRows.length > 0 && adBreak.campaignRows.length === 0
    );

    const campaignUseCounts = new Map();
    let filled = 0;
    let partial = 0;
    let skipped = 0;
    let inserted = 0;
    let deletedFillers = 0;
    let trimmedFillers = 0;
    const messages = [];

    for (const adBreak of breaks) {
      const screenDuration = sumDuration(adBreak.fillerRows);
      const spots = await selectSpotsForBreak(db, adBreak.date, screenDuration, stationId, campaignUseCounts);
      const adsDuration = sumDuration(spots);

      if (spots.length === 0 || adsDuration <= EPSILON) {
        skipped += 1;
        messages.push(`${adBreak.date} ${adBreak.start_time}: skipped, no active campaign spot fits ${formatDuration(screenDuration)}.`);
        continue;
      }

      const insertedRows = [];
      let offset = 0;
      for (const spot of spots) {
        const insertedId = await insertAdQueueEntry(db, spot, adBreak.firstScheduledAt, offset, adBreak.firstPriority);
        insertedRows.push({
          id: insertedId,
          play_duration: Number(spot.play_duration || 0),
          stretch_rate: 1
        });
        campaignUseCounts.set(spot.campaign_id, (campaignUseCounts.get(spot.campaign_id) || 0) + 1);
        inserted += 1;
        offset += Number(spot.play_duration || 0);
      }

      const adjustment = await consumeFillers(db, adBreak.fillerRows, adsDuration);
      deletedFillers += adjustment.deleted;
      trimmedFillers += adjustment.trimmed;

      const remainingFillerIds = adBreak.fillerRows
        .filter((row) => !adjustment.deletedIds.has(row.id))
        .map((row) => row.id);
      await rescheduleBreakRows(db, [
        ...insertedRows.map((row) => row.id),
        ...remainingFillerIds
      ], adBreak.firstScheduledAt);

      if (remainingFillerIds.length === 0) {
        filled += 1;
      } else {
        partial += 1;
      }

      messages.push(`${adBreak.date} ${adBreak.start_time}: inserted ${spots.length} spot(s), ${formatDuration(adsDuration)} / ${formatDuration(screenDuration)}.`);
    }

    await db.query('COMMIT');
    return {
      screens: breaks.length,
      filled,
      partial,
      skipped,
      inserted,
      deletedFillers,
      trimmedFillers,
      messages: messages.slice(-80)
    };
  } catch (error) {
    await db.query('ROLLBACK');
    throw error;
  }
}

async function listAdSchedulerRows(db, timezone, { fromLocal, toLocal, days = 7 }) {
  const rangeSql = fromLocal && toLocal
    ? `
      q.scheduled_at >= ($2::timestamp AT TIME ZONE $1)
      AND q.scheduled_at < (($3::timestamp + INTERVAL '1 hour') AT TIME ZONE $1)
    `
    : `
      q.scheduled_at >= (CURRENT_DATE::timestamp AT TIME ZONE $1)
      AND q.scheduled_at < ((CURRENT_DATE + $2::integer)::timestamp AT TIME ZONE $1)
    `;
  const values = fromLocal && toLocal ? [timezone, fromLocal, toLocal] : [timezone, days];

  const { rows } = await db.query(
    `
    SELECT
      q.id,
      q.track_id,
      q.cue_in::double precision AS cue_in,
      q.cue_out::double precision AS cue_out,
      q.stretch_rate::double precision AS stretch_rate,
      q.priority,
      q.fixed_time,
      q.scheduled_at,
      to_char(q.scheduled_at AT TIME ZONE $1, 'YYYY-MM-DD') AS date,
      EXTRACT(HOUR FROM q.scheduled_at AT TIME ZONE $1)::integer AS hour,
      to_char(q.scheduled_at AT TIME ZONE $1, 'HH24:MI:SS') AS start_time,
      COALESCE(t.title, '') AS title,
      t.track_type_id,
      ct.id AS campaign_track_id,
      cp.id AS campaign_id,
      cp.active AS campaign_active,
      q.played,
      CASE
        WHEN cp.id IS NOT NULL
          AND cp.active = TRUE
          AND (cp.start_date IS NULL OR cp.start_date <= (q.scheduled_at AT TIME ZONE $1)::date)
          AND (cp.end_date IS NULL OR cp.end_date >= (q.scheduled_at AT TIME ZONE $1)::date)
          AND (cp.total_broadcasts <= 0 OR cp.broadcast_count < cp.total_broadcasts)
        THEN TRUE
        ELSE FALSE
      END AS active_campaign
    FROM queue q
    LEFT JOIN tracks t ON t.id = q.track_id
    LEFT JOIN campaign_tracks ct ON ct.track_id = q.track_id
    LEFT JOIN campaigns cp ON cp.id = ct.campaign_id
    WHERE ${rangeSql}
      AND q.played = FALSE
    ORDER BY q.scheduled_at, q.priority, q.id
    `,
    values
  );

  return rows.map((row) => ({
    ...row,
    play_duration: playDuration(row),
    isFiller: Number(row.track_type_id) === FILLER_TRACK_TYPE_ID,
    isCampaign: row.campaign_track_id != null,
    isActiveCampaign: row.active_campaign === true
  }));
}

function groupAdBreaks(rows) {
  const breaks = [];
  let current = null;

  for (const row of rows) {
    const isBreakRow = row.isFiller || row.isCampaign;
    if (!isBreakRow) {
      if (current) breaks.push(finalizeBreak(current));
      current = null;
      continue;
    }

    if (!current) {
      current = { rows: [] };
    }
    current.rows.push(row);
  }

  if (current) breaks.push(finalizeBreak(current));
  return breaks;
}

function finalizeBreak(adBreak) {
  const first = adBreak.rows[0];
  const fillerRows = adBreak.rows.filter((row) => row.isFiller);
  const campaignRows = adBreak.rows.filter((row) => row.isCampaign);
  const activeCampaignRows = adBreak.rows.filter((row) => row.isActiveCampaign);
  return {
    ...adBreak,
    date: first.date,
    hour: first.hour,
    start_time: first.start_time,
    firstScheduledAt: first.scheduled_at,
    firstPriority: first.priority || 0,
    fillerRows,
    campaignRows,
    activeCampaignRows
  };
}

function summarizeBreaks(breaks) {
  return breaks.map((adBreak, index) => {
    const adCount = adBreak.campaignRows.length;
    const fillerCount = adBreak.fillerRows.length;
    const status = adCount > 0 && fillerCount === 0
      ? 'filled'
      : adCount > 0
        ? 'partial'
        : 'empty';
    return {
      id: `${adBreak.date}-${adBreak.hour}-${index}`,
      date: adBreak.date,
      hour: adBreak.hour,
      start_time: adBreak.start_time,
      status,
      ad_count: adCount,
      filler_count: fillerCount,
      ad_duration: sumDuration(adBreak.campaignRows),
      filler_duration: sumDuration(adBreak.fillerRows)
    };
  });
}

async function selectSpotsForBreak(db, date, maxDuration, stationId, campaignUseCounts) {
  const { rows } = await db.query(
    `
    SELECT
      cp.id AS campaign_id,
      cp.name AS campaign_name,
      ct.track_id,
      ct.position,
      t.title,
      t.cue_in::double precision AS cue_in,
      COALESCE(t.cue_out, t.duration)::double precision AS cue_out,
      CASE
        WHEN t.cue_out IS NOT NULL AND t.cue_out > t.cue_in THEN t.cue_out - t.cue_in
        ELSE GREATEST(t.duration - t.cue_in, 0)
      END::double precision AS play_duration,
      COALESCE(existing.scheduled_count, 0)::integer AS scheduled_count
    FROM campaigns cp
    JOIN campaign_tracks ct ON ct.campaign_id = cp.id
    JOIN tracks t ON t.id = ct.track_id
    LEFT JOIN (
      SELECT ct2.campaign_id, COUNT(*) AS scheduled_count
      FROM queue q2
      JOIN campaign_tracks ct2 ON ct2.track_id = q2.track_id
      WHERE q2.played = FALSE
      GROUP BY ct2.campaign_id
    ) existing ON existing.campaign_id = cp.id
    WHERE cp.active = TRUE
      AND t.active = TRUE
      AND ($2::integer IS NULL OR cp.station_id IS NULL OR cp.station_id = $2)
      AND (cp.start_date IS NULL OR cp.start_date <= $1::date)
      AND (cp.end_date IS NULL OR cp.end_date >= $1::date)
      AND (t.start_date IS NULL OR t.start_date <= $1::date)
      AND (t.end_date IS NULL OR t.end_date >= $1::date)
      AND (cp.total_broadcasts <= 0 OR cp.broadcast_count < cp.total_broadcasts)
    ORDER BY COALESCE(existing.scheduled_count, 0), cp.broadcast_count, cp.id, ct.position
    `,
    [date, stationId]
  );

  const campaigns = new Map();
  for (const row of rows) {
    if (Number(row.play_duration || 0) <= EPSILON) continue;
    if (!campaigns.has(row.campaign_id)) campaigns.set(row.campaign_id, []);
    campaigns.get(row.campaign_id).push(row);
  }

  const campaignGroups = [...campaigns.entries()]
    .map(([campaignId, tracks]) => ({
      campaignId,
      tracks,
      useCount: campaignUseCounts.get(campaignId) || 0,
      scheduledCount: Number(tracks[0]?.scheduled_count || 0)
    }))
    .sort((a, b) =>
      (a.scheduledCount + a.useCount) - (b.scheduledCount + b.useCount)
      || a.campaignId - b.campaignId
    );

  const selectedCampaignIds = new Set();
  let remaining = maxDuration;
  const selected = [];

  for (const group of campaignGroups) {
    if (selectedCampaignIds.has(group.campaignId)) continue;

    const start = group.useCount % group.tracks.length;
    const rotated = [...group.tracks.slice(start), ...group.tracks.slice(0, start)];
    const spot = rotated.find((candidate) => Number(candidate.play_duration || 0) <= remaining + EPSILON);
    if (!spot) continue;
    selected.push(spot);
    selectedCampaignIds.add(group.campaignId);
    remaining -= Number(spot.play_duration || 0);
    if (remaining <= EPSILON) break;
  }

  return selected;
}

async function insertAdQueueEntry(db, spot, firstScheduledAt, offsetSeconds, priority) {
  const { rows } = await db.query(
    `
    INSERT INTO queue (track_id, cue_in, cue_out, stretch_rate, played, priority, fixed_time, scheduled_at)
    VALUES ($1, $2, $3, 1, FALSE, $4, FALSE, $5::timestamptz + ($6::double precision * INTERVAL '1 second'))
    RETURNING id
    `,
    [spot.track_id, spot.cue_in, spot.cue_out, priority, firstScheduledAt, offsetSeconds]
  );
  return rows[0].id;
}

async function consumeFillers(db, fillerRows, adDuration) {
  let remaining = adDuration;
  const deletedIds = new Set();
  let deleted = 0;
  let trimmed = 0;

  const smallestFirst = [...fillerRows].sort((a, b) =>
    a.play_duration - b.play_duration || a.id - b.id
  );

  for (const filler of smallestFirst) {
    if (remaining <= EPSILON) break;

    if (remaining >= filler.play_duration - EPSILON) {
      await db.query('DELETE FROM queue WHERE id = $1', [filler.id]);
      deletedIds.add(filler.id);
      deleted += 1;
      remaining -= filler.play_duration;
      continue;
    }

    const stretchRate = Number(filler.stretch_rate || 1);
    const newCueOut = Number(filler.cue_out || 0) - remaining * stretchRate;
    await db.query(
      'UPDATE queue SET cue_out = $2, updated_at = NOW() WHERE id = $1',
      [filler.id, Math.max(Number(filler.cue_in || 0), newCueOut)]
    );
    trimmed += 1;
    remaining = 0;
  }

  return { deleted, deletedIds, trimmed };
}

async function rescheduleBreakRows(db, ids, firstScheduledAt) {
  let offset = 0;
  for (let index = 0; index < ids.length; index += 1) {
    const { rows } = await db.query(
      'SELECT cue_in, cue_out, stretch_rate FROM queue WHERE id = $1',
      [ids[index]]
    );
    if (!rows[0]) continue;

    await db.query(
      `
      UPDATE queue
      SET scheduled_at = $2::timestamptz + ($3::double precision * INTERVAL '1 second'),
          priority = $4,
          updated_at = NOW()
      WHERE id = $1
      `,
      [ids[index], firstScheduledAt, offset, index + 1]
    );

    offset += playDuration(rows[0]);
  }
}

function sumDuration(rows) {
  return rows.reduce((total, row) => total + Number(row.play_duration || 0), 0);
}

function playDuration(row) {
  const cueIn = Number(row.cue_in || 0);
  const cueOut = Number(row.cue_out || 0);
  const stretchRate = Number(row.stretch_rate || 1);
  return Math.max(0, (cueOut - cueIn) / stretchRate);
}

function hourBoundary(date, hour) {
  return `${date} ${String(hour).padStart(2, '0')}:00:00`;
}

function formatDuration(seconds) {
  const total = Math.round(Number(seconds || 0));
  const minutes = Math.floor(total / 60);
  return `${minutes}:${String(total % 60).padStart(2, '0')}`;
}
