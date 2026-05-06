import { randomUUID } from 'node:crypto';
import { withDatabase } from '../db/client.js';
import {
  countQueueInPeriod,
  currentHourBoundaryInTimezone,
  findTrackForSlot,
  getConfiguredTimezone,
  getScheduleForHour,
  getTrackById,
  insertFixedQueueEntry,
  insertQueueEntry,
  insertQueueEntryWithCutoff,
  listEventsForHour,
  listSlotsForGenerator
} from '../repositories/playlists.js';

const MAX_MESSAGES = 500;
const HOUR_LIMIT_SECONDS = 3599.999;
const DAY_KEYS = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];
const jobs = new Map();

export function startQueueGeneration(databaseConfig, options) {
  const job = {
    id: randomUUID(),
    status: 'queued',
    fromDate: options.fromDate,
    fromHour: options.fromHour,
    toDate: options.toDate,
    toHour: options.toHour,
    total: 0,
    processed: 0,
    created: 0,
    skipped: 0,
    skippedHours: 0,
    current: '',
    messages: [],
    startedAt: new Date().toISOString(),
    finishedAt: null
  };

  jobs.set(job.id, job);
  setTimeout(() => {
    runQueueGenerationJob(databaseConfig, job).catch((error) => {
      job.status = 'failed';
      job.current = '';
      job.finishedAt = new Date().toISOString();
      addMessage(job, `Failed: ${error.message}`);
    });
  }, 0);

  return serializeJob(job);
}

export function getQueueGenerationJob(id) {
  const job = jobs.get(id);
  return job ? serializeJob(job) : null;
}

class QueueGenerator {
  constructor(db, job, options) {
    this.db = db;
    this.job = job;
    this.fromDate = options.fromDate;
    this.fromHour = options.fromHour;
    this.toDate = options.toDate;
    this.toHour = options.toHour;
    this.timezone = 'Europe/Paris';
  }

  async generate() {
    this.timezone = await getConfiguredTimezone(this.db);
    await this.validateFutureRange();

    const hours = buildHours(this.fromDate, this.fromHour, this.toDate, this.toHour);
    this.job.total = hours.length;
    addMessage(this.job, `Generating ${hours.length} hour${hours.length === 1 ? '' : 's'} in ${this.timezone}.`);

    await this.db.query('BEGIN');
    try {
      const existing = await countQueueInPeriod(
        this.db,
        hourBoundary(this.fromDate, this.fromHour),
        hourBoundary(this.toDate, this.toHour),
        this.timezone
      );
      if (existing > 0) {
        const error = new Error('Queue already exists in this period. Generation cancelled.');
        error.statusCode = 409;
        throw error;
      }

      for (const hourInfo of hours) {
        await this.generateHour(hourInfo);
        this.job.processed += 1;
      }

      await this.db.query('COMMIT');
    } catch (error) {
      await this.db.query('ROLLBACK');
      throw error;
    }

    this.job.status = 'completed';
    this.job.current = '';
    this.job.finishedAt = new Date().toISOString();
    addMessage(this.job, `Completed. Created ${this.job.created}, skipped ${this.job.skipped}.`);
  }

  async validateFutureRange() {
    const currentHour = await currentHourBoundaryInTimezone(this.db, this.timezone);
    const requestedHour = hourBoundary(this.fromDate, this.fromHour);
    if (requestedHour < currentHour) {
      throw new Error(`Generation must start from the current hour or later. Current hour is ${currentHour}.`);
    }
  }

  async generateHour(hourInfo) {
    const label = `${hourInfo.date} ${pad(hourInfo.hour)}:00`;
    this.job.current = label;

    const schedule = await getScheduleForHour(this.db, hourInfo.dayKey, hourInfo.hour);
    const eventRows = await listEventsForHour(this.db, hourInfo.date, hourInfo.dayKey, hourInfo.hour);

    // Sort events by their scheduled position within the hour
    const pendingEvents = [...eventRows].sort((a, b) => {
      const ta = a.minute * 60 + a.second;
      const tb = b.minute * 60 + b.second;
      return ta !== tb ? ta - tb : b.priority - a.priority;
    });
    let eventIdx = 0;
    let offsetSeconds = 0;

    if (!schedule) {
      this.job.skippedHours += 1;
      addMessage(this.job, `Skipped hour ${label}: no schedule.`);
    } else {
      const slots = await listSlotsForGenerator(this.db, schedule.template_id);
      if (slots.length === 0) {
        this.job.skippedHours += 1;
        addMessage(this.job, `Skipped hour ${label}: template ${schedule.template_name} has no slots.`);
      } else {
        let createdForHour = 0;

        // Fire fixed events scheduled at the very start of the hour (before any music)
        while (eventIdx < pendingEvents.length) {
          const ev = pendingEvents[eventIdx];
          const evTime = ev.minute * 60 + ev.second;
          if (!ev.is_fixed || evTime > offsetSeconds) break;
          const dur = await this.insertEventAction(ev, hourInfo, evTime);
          offsetSeconds = Math.max(offsetSeconds, evTime + dur);
          eventIdx++;
        }

        for (const slot of slots) {
          if (offsetSeconds >= HOUR_LIMIT_SECONDS) break;

          // Insert floating events whose scheduled time has been reached
          while (eventIdx < pendingEvents.length) {
            const ev = pendingEvents[eventIdx];
            const evTime = ev.minute * 60 + ev.second;
            if (ev.is_fixed || evTime > offsetSeconds) break;
            const dur = await this.insertEventAction(ev, hourInfo, offsetSeconds);
            offsetSeconds += dur;
            eventIdx++;
          }

          // Find the nearest upcoming fixed event to determine if we must cut this slot short
          let cutoffSeconds = Infinity;
          for (let i = eventIdx; i < pendingEvents.length; i++) {
            if (pendingEvents[i].is_fixed) {
              const t = pendingEvents[i].minute * 60 + pendingEvents[i].second;
              if (t >= offsetSeconds) { cutoffSeconds = t; break; }
            }
          }

          const scheduledAtLocal = localTimestamp(hourInfo.date, hourInfo.hour, offsetSeconds);
          const track = await findTrackForSlot(this.db, slot, scheduledAtLocal, this.timezone);

          if (!track) {
            this.job.skipped += 1;
            const prot = `prot. track ${slot.track_protection}s / artiste ${slot.artist_protection}s`;
            addMessage(this.job, `✗ Skip [${slot.label}] – aucune piste disponible (${prot})`, 'skip');
            continue;
          }

          const playDuration = Number(track.play_duration || 0);

          if (Number.isFinite(cutoffSeconds) && offsetSeconds + playDuration > cutoffSeconds) {
            // Track would extend past the fixed event — cut it short
            const maxDuration = cutoffSeconds - offsetSeconds;
            if (maxDuration > 0) {
              await insertQueueEntryWithCutoff(this.db, track, maxDuration, scheduledAtLocal, this.timezone, slot.position);
              createdForHour++;
              this.job.created++;
              const who = track.artist_name ? `${track.artist_name} – ${track.title}` : track.title;
              addMessage(this.job, `~ ${scheduledAtLocal}  ${who}  (coupé à ${formatDuration(maxDuration)} / ${formatDuration(playDuration)})`, 'cut');
            }
            offsetSeconds = cutoffSeconds;

            // Insert all events at or before the cutoff time, advancing offsetSeconds as we go
            while (eventIdx < pendingEvents.length) {
              const ev = pendingEvents[eventIdx];
              const evTime = ev.minute * 60 + ev.second;
              if (evTime > cutoffSeconds) break;
              const eventAtSeconds = ev.is_fixed ? evTime : offsetSeconds;
              const dur = await this.insertEventAction(ev, hourInfo, eventAtSeconds);
              offsetSeconds = Math.max(offsetSeconds, eventAtSeconds + dur);
              eventIdx++;
            }
          } else {
            await insertQueueEntry(this.db, track, scheduledAtLocal, this.timezone, slot.position);
            offsetSeconds += playDuration;
            createdForHour++;
            this.job.created++;
            const who = track.artist_name ? `${track.artist_name} – ${track.title}` : track.title;
            addMessage(this.job, `→ ${scheduledAtLocal}  ${who}  (${formatDuration(playDuration)})`, 'track');
          }
        }

        addMessage(
          this.job,
          `${label}: ${createdForHour} track${createdForHour === 1 ? '' : 's'} from ${schedule.template_name}.`
        );
      }
    }

    // Insert any events that remain after all slots (or when there is no schedule)
    while (eventIdx < pendingEvents.length) {
      const ev = pendingEvents[eventIdx];
      const evTime = ev.minute * 60 + ev.second;
      const eventAtSeconds = ev.is_fixed ? evTime : offsetSeconds;
      const dur = await this.insertEventAction(ev, hourInfo, eventAtSeconds);
      offsetSeconds = Math.max(offsetSeconds, eventAtSeconds + dur);
      eventIdx++;
    }
  }

  async insertEventAction(ev, hourInfo, atSeconds) {
    let totalDuration = 0;

    if (ev.action_type === 2) {
      const track = await getTrackById(this.db, ev.track_id);
      if (!track) return 0;
      const scheduledAtLocal = localTimestamp(hourInfo.date, hourInfo.hour, atSeconds);
      await insertFixedQueueEntry(this.db, track, ev.priority, scheduledAtLocal, this.timezone);
      this.job.created++;
      const dur = Number(track.cue_out || 0) - Number(track.cue_in || 0);
      const who = track.artist_name ? `${track.artist_name} – ${track.title}` : track.title;
      const timing = ev.is_fixed ? 'fixe' : 'flottant';
      addMessage(this.job, `★ ${scheduledAtLocal}  [${ev.name || 'Event'} – ${timing}]  ${who}  (${formatDuration(dur)})`, 'event');
      totalDuration = dur;
    } else if (ev.action_type === 1) {
      const slots = await listSlotsForGenerator(this.db, ev.template_id);
      for (const slot of slots) {
        const scheduledAtLocal = localTimestamp(hourInfo.date, hourInfo.hour, atSeconds + totalDuration);
        const track = await findTrackForSlot(this.db, slot, scheduledAtLocal, this.timezone);
        if (!track) {
          this.job.skipped++;
          addMessage(this.job, `✗ Skip event [${ev.name || 'Event'}] slot [${slot.label}] – aucune piste disponible`, 'skip');
          continue;
        }
        await insertFixedQueueEntry(this.db, track, ev.priority, scheduledAtLocal, this.timezone);
        this.job.created++;
        const dur = Number(track.play_duration || 0);
        const who = track.artist_name ? `${track.artist_name} – ${track.title}` : track.title;
        const timing = ev.is_fixed ? 'fixe' : 'flottant';
        addMessage(this.job, `★ ${scheduledAtLocal}  [${ev.name || 'Event'} – ${timing}]  ${who}  (${formatDuration(dur)})`, 'event');
        totalDuration += dur;
      }
    }

    return totalDuration;
  }
}

async function runQueueGenerationJob(databaseConfig, job) {
  job.status = 'running';
  addMessage(
    job,
    `Starting generation from ${job.fromDate} ${pad(job.fromHour)}:00 to ${job.toDate} ${pad(job.toHour)}:00.`
  );

  await withDatabase(databaseConfig, async (db) => {
    const generator = new QueueGenerator(db, job, {
      fromDate: job.fromDate,
      fromHour: job.fromHour,
      toDate: job.toDate,
      toHour: job.toHour
    });
    await generator.generate();
  });
}

function buildHours(fromDate, fromHour, toDate, toHour) {
  const dates = buildDates(fromDate, toDate);
  const hours = [];

  for (const date of dates) {
    const startHour = date === fromDate ? fromHour : 0;
    const endHour = date === toDate ? toHour : 23;

    for (let hour = startHour; hour <= endHour; hour += 1) {
      hours.push({
        date,
        hour,
        dayKey: DAY_KEYS[dateToUtcDate(date).getUTCDay()]
      });
    }
  }

  return hours;
}

function buildDates(fromDate, toDate) {
  const dates = [];
  let cursor = dateToUtcDate(fromDate);
  const end = dateToUtcDate(toDate);

  while (cursor <= end) {
    dates.push(formatUtcDate(cursor));
    cursor = new Date(cursor.getTime() + 24 * 60 * 60 * 1000);
  }

  return dates;
}

function dateToUtcDate(value) {
  const [year, month, day] = value.split('-').map(Number);
  return new Date(Date.UTC(year, month - 1, day));
}

function formatUtcDate(date) {
  return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}`;
}

function localTimestamp(date, hour, offsetSeconds) {
  const totalSeconds = hour * 3600 + offsetSeconds;
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  if (h >= 24) {
    const nextDay = new Date(dateToUtcDate(date).getTime() + 24 * 60 * 60 * 1000);
    return `${formatUtcDate(nextDay)} ${pad(h - 24)}:${pad(m)}:${formatSeconds(s)}`;
  }
  return `${date} ${pad(h)}:${pad(m)}:${formatSeconds(s)}`;
}

function hourBoundary(date, hour) {
  return `${date} ${pad(hour)}:00:00`;
}

function formatSeconds(value) {
  const whole = Math.floor(value);
  const milliseconds = Math.floor((value - whole) * 1000);
  if (milliseconds === 0) return pad(whole);
  return `${pad(whole)}.${String(milliseconds).padStart(3, '0')}`;
}

function pad(value) {
  return String(value).padStart(2, '0');
}

function addMessage(job, message, type = 'info') {
  job.messages.push({ at: new Date().toISOString(), message, type });
  if (job.messages.length > MAX_MESSAGES) {
    job.messages.splice(0, job.messages.length - MAX_MESSAGES);
  }
}

function formatDuration(seconds) {
  const s = Math.floor(seconds);
  const m = Math.floor(s / 60);
  return `${m}:${pad(s % 60)}`;
}

function serializeJob(job) {
  return {
    ...job,
    messages: [...job.messages]
  };
}
