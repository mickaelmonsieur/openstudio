import { randomUUID } from 'node:crypto';
import { withDatabase } from '../db/client.js';
import {
  countQueueInPeriod,
  currentHourBoundaryInTimezone,
  findTrackForSlot,
  getConfiguredTimezone,
  getScheduleForHour,
  insertQueueEntry,
  listSlotsForGenerator
} from '../repositories/playlists.js';

const MAX_MESSAGES = 80;
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
    if (!schedule) {
      this.job.skippedHours += 1;
      addMessage(this.job, `Skipped hour ${label}: no schedule.`);
      return;
    }

    const slots = await listSlotsForGenerator(this.db, schedule.template_id);
    if (slots.length === 0) {
      this.job.skippedHours += 1;
      addMessage(this.job, `Skipped hour ${label}: template ${schedule.template_name} has no slots.`);
      return;
    }

    let offsetSeconds = 0;
    let createdForHour = 0;

    for (const slot of slots) {
      if (offsetSeconds >= HOUR_LIMIT_SECONDS) break;

      const remainingSeconds = HOUR_LIMIT_SECONDS - offsetSeconds;
      const scheduledAtLocal = localTimestamp(hourInfo.date, hourInfo.hour, offsetSeconds);
      const track = await findTrackForSlot(this.db, slot, scheduledAtLocal, this.timezone, remainingSeconds);

      if (!track) {
        this.job.skipped += 1;
        continue;
      }

      const playDuration = Number(track.play_duration || 0);
      if (offsetSeconds + playDuration > HOUR_LIMIT_SECONDS) {
        break;
      }

      await insertQueueEntry(this.db, track, scheduledAtLocal, this.timezone, slot.position);
      offsetSeconds += playDuration;
      createdForHour += 1;
      this.job.created += 1;
    }

    addMessage(
      this.job,
      `${label}: ${createdForHour} track${createdForHour === 1 ? '' : 's'} from ${schedule.template_name}.`
    );
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

function addMessage(job, message) {
  job.messages.push({
    at: new Date().toISOString(),
    message
  });

  if (job.messages.length > MAX_MESSAGES) {
    job.messages.splice(0, job.messages.length - MAX_MESSAGES);
  }
}

function serializeJob(job) {
  return {
    ...job,
    messages: [...job.messages]
  };
}
