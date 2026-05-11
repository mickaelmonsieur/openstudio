import { withDatabase } from '../db/client.js';
import {
  listSectors,
  countAdvertisers, listAdvertisers, getAdvertiser, createAdvertiser, updateAdvertiser, deleteAdvertiser,
  countContacts, listContacts, getContact, createContact, updateContact, deleteContact,
  countCampaigns, listCampaigns, getCampaign, createCampaign, updateCampaign, deleteCampaign,
  listCampaignBroadcastHours, replaceCampaignBroadcastHours,
  listCampaignCalendarHours, replaceCampaignCalendarHours,
  countCampaignTracks, listCampaignTracks, getCampaignTrack, createCampaignTrack, updateCampaignTrack, deleteCampaignTrack
} from '../repositories/advertising.js';

const ADV_LIMIT = 50;
const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1, 10) || 1);
  const limit = Math.min(200, Math.max(1, parseInt(query.limit || ADV_LIMIT, 10) || ADV_LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

function parseSearch(query) {
  return String(query.q || '').trim().slice(0, 120);
}

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function parseOptionalId(value) {
  if (value === undefined || value === null || value === '') return null;
  return parseId(value);
}

function parseActiveFilter(value) {
  if (value === undefined || value === null || value === '') return null;
  if (value === 'true' || value === '1') return true;
  if (value === 'false' || value === '0') return false;
  return null;
}

function str(val, max) {
  const s = String(val || '').trim();
  return s.length > max ? s.slice(0, max) : s;
}

function optDate(val) {
  const s = String(val || '').trim();
  return s || null;
}

function asyncRoute(handler) {
  return async (req, res) => {
    try {
      await handler(req, res);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  };
}

function validateAdvertiser(data) {
  const name = String(data?.name || '').trim();
  if (!name) return { ok: false, error: 'Name is required.' };
  if (name.length > 255) return { ok: false, error: 'Name is too long (max 255).' };
  const sector_id = parseId(data?.sector_id);
  if (!sector_id) return { ok: false, error: 'Sector is required.' };
  return {
    ok: true,
    value: {
      name,
      sector_id,
      address:     String(data?.address  || '').trim() || null,
      vat_number:  str(data?.vat_number, 32) || null,
      notes:       String(data?.notes    || '').trim() || null,
      active:      Boolean(data?.active ?? true),
      client_since: optDate(data?.client_since)
    }
  };
}

function validateContact(data) {
  const advertiser_id = parseId(data?.advertiser_id);
  if (!advertiser_id) return { ok: false, error: 'Advertiser is required.' };
  const name = String(data?.name || '').trim();
  if (!name) return { ok: false, error: 'Name is required.' };
  if (name.length > 128) return { ok: false, error: 'Name is too long (max 128).' };
  return {
    ok: true,
    value: {
      advertiser_id,
      name,
      role:            str(data?.role,  64) || null,
      phone:           str(data?.phone, 32) || null,
      email:           str(data?.email, 128) || null,
      primary_contact: Boolean(data?.primary_contact),
      notes:           String(data?.notes || '').trim() || null
    }
  };
}

function validateCampaign(data) {
  const advertiser_id = parseId(data?.advertiser_id);
  if (!advertiser_id) return { ok: false, error: 'Advertiser is required.' };
  const name = str(data?.name, 255);
  if (!name) return { ok: false, error: 'Name is required.' };
  const start_date = optDate(data?.start_date);
  const end_date = optDate(data?.end_date);
  if (!start_date || !DATE_RE.test(start_date)) return { ok: false, error: 'Start date is required.' };
  if (!end_date || !DATE_RE.test(end_date)) return { ok: false, error: 'End date is required.' };
  if (end_date < start_date) return { ok: false, error: 'End date must be after start date.' };
  return {
    ok: true,
    value: {
      advertiser_id,
      name,
      station_id:       data?.station_id ? parseId(data.station_id) : null,
      total_broadcasts: Math.max(0, parseInt(data?.total_broadcasts || 0, 10) || 0),
      max_broadcasts_per_day: Math.max(0, parseInt(data?.max_broadcasts_per_day || 0, 10) || 0),
      min_broadcast_gap_minutes: Math.max(0, parseInt(data?.min_broadcast_gap_minutes || 0, 10) || 0),
      active:           Boolean(data?.active ?? true),
      start_date,
      end_date
    }
  };
}

function validateCampaignTrack(data) {
  const campaign_id = parseId(data?.campaign_id);
  if (!campaign_id) return { ok: false, error: 'Campaign is required.' };
  const track_id = parseId(data?.track_id);
  if (!track_id) return { ok: false, error: 'Track is required.' };
  const pos = parseInt(data?.position || 0, 10);
  const screen_position = parseScreenPosition(data?.screen_position);
  if (screen_position === null) return { ok: false, error: 'Screen position is invalid.' };
  return {
    ok: true,
    value: { campaign_id, track_id, position: pos > 0 ? pos : null, screen_position }
  };
}

function validateBroadcastHours(data) {
  if (!Array.isArray(data?.hours)) {
    return { ok: false, error: 'Broadcast hours are required.' };
  }

  const seen = new Set();
  const hours = [];
  for (const item of data.hours) {
    const iso_weekday = Number(item?.iso_weekday);
    const hour = Number(item?.hour);
    if (!Number.isInteger(iso_weekday) || iso_weekday < 1 || iso_weekday > 7) {
      return { ok: false, error: 'Broadcast day is invalid.' };
    }
    if (!Number.isInteger(hour) || hour < 0 || hour > 23) {
      return { ok: false, error: 'Broadcast hour is invalid.' };
    }

    const key = `${iso_weekday}:${hour}`;
    if (seen.has(key)) continue;
    seen.add(key);
    hours.push({ iso_weekday, hour });
  }

  return { ok: true, value: hours };
}

function validateCalendarHours(data) {
  if (!Array.isArray(data?.dates)) {
    return { ok: false, error: 'Calendar dates are required.' };
  }

  const seen = new Set();
  const hours = [];
  for (const dateRule of data.dates) {
    const broadcast_date = optDate(dateRule?.broadcast_date);
    if (!broadcast_date || !DATE_RE.test(broadcast_date)) {
      return { ok: false, error: 'Calendar date is invalid.' };
    }
    if (!Array.isArray(dateRule?.hours)) {
      return { ok: false, error: 'Calendar hours are invalid.' };
    }

    for (const value of dateRule.hours) {
      const hour = Number(value);
      if (!Number.isInteger(hour) || hour < 0 || hour > 23) {
        return { ok: false, error: 'Calendar hour is invalid.' };
      }

      const key = `${broadcast_date}:${hour}`;
      if (seen.has(key)) continue;
      seen.add(key);
      hours.push({ broadcast_date, hour, active: true });
    }
  }

  return { ok: true, value: hours };
}

function parseScreenPosition(value) {
  const screenPosition = Number(value ?? 1);
  return Number.isInteger(screenPosition) && screenPosition >= 0 && screenPosition <= 2
    ? screenPosition
    : null;
}

export function registerAdvertisingRoutes(app, getDatabaseConfig) {
  // Sectors (read-only)
  app.get('/api/sectors', asyncRoute(async (_req, res) => {
    const rows = await withDatabase(getDatabaseConfig(), listSectors);
    res.json({ rows });
  }));

  // Advertisers
  app.get('/api/advertisers', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const search = parseSearch(req.query);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countAdvertisers(db, search), listAdvertisers(db, { limit, offset, search })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/advertisers', asyncRoute(async (req, res) => {
    const r = validateAdvertiser(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createAdvertiser(db, r.value));
    res.status(201).json({ row });
  }));

  app.put('/api/advertisers/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateAdvertiser(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => updateAdvertiser(db, id, r.value));
    if (!row) { res.status(404).json({ error: 'Advertiser not found.' }); return; }
    res.json({ row });
  }));

  app.delete('/api/advertisers/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteAdvertiser(db, id));
    if (!deleted) { res.status(404).json({ error: 'Advertiser not found.' }); return; }
    res.status(204).send();
  }));

  // Contacts
  app.get('/api/contacts', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const search = parseSearch(req.query);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countContacts(db, search), listContacts(db, { limit, offset, search })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/contacts', asyncRoute(async (req, res) => {
    const r = validateContact(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createContact(db, r.value));
    res.status(201).json({ row });
  }));

  app.put('/api/contacts/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateContact(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => updateContact(db, id, r.value));
    if (!row) { res.status(404).json({ error: 'Contact not found.' }); return; }
    res.json({ row });
  }));

  app.delete('/api/contacts/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteContact(db, id));
    if (!deleted) { res.status(404).json({ error: 'Contact not found.' }); return; }
    res.status(204).send();
  }));

  // Campaigns
  app.get('/api/campaigns', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const search = parseSearch(req.query);
    const advertiserId = parseOptionalId(req.query.advertiser_id);
    const active = parseActiveFilter(req.query.active);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([
        countCampaigns(db, { search, advertiserId, active }),
        listCampaigns(db, { limit, offset, search, advertiserId, active })
      ])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/campaigns', asyncRoute(async (req, res) => {
    const r = validateCampaign(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createCampaign(db, r.value));
    res.status(201).json({ row });
  }));

  app.get('/api/campaigns/:id/broadcast-hours', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const campaign = await getCampaign(db, id);
      if (!campaign) return null;
      const hours = await listCampaignBroadcastHours(db, id);
      return { campaign, hours };
    });
    if (!payload) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.json(payload);
  }));

  app.put('/api/campaigns/:id/broadcast-hours', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateBroadcastHours(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }

    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const campaign = await getCampaign(db, id);
      if (!campaign) return null;
      const hours = await replaceCampaignBroadcastHours(db, id, r.value);
      return { campaign, hours };
    });
    if (!payload) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.json(payload);
  }));

  app.get('/api/campaigns/:id/calendar-hours', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const campaign = await getCampaign(db, id);
      if (!campaign) return null;
      const hours = await listCampaignCalendarHours(db, id);
      return { campaign, hours };
    });
    if (!payload) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.json(payload);
  }));

  app.put('/api/campaigns/:id/calendar-hours', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateCalendarHours(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }

    const payload = await withDatabase(getDatabaseConfig(), async (db) => {
      const campaign = await getCampaign(db, id);
      if (!campaign) return null;
      const hours = await replaceCampaignCalendarHours(db, id, r.value);
      return { campaign, hours };
    });
    if (!payload) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.json(payload);
  }));

  app.put('/api/campaigns/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateCampaign(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => updateCampaign(db, id, r.value));
    if (!row) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.json({ row });
  }));

  app.delete('/api/campaigns/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteCampaign(db, id));
    if (!deleted) { res.status(404).json({ error: 'Campaign not found.' }); return; }
    res.status(204).send();
  }));

  // Campaign Tracks
  app.get('/api/campaign-tracks/options', asyncRoute(async (_req, res) => {
    const campaigns = await withDatabase(getDatabaseConfig(), listCampaigns);
    res.json({ campaigns });
  }));

  app.get('/api/campaign-tracks', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const search = parseSearch(req.query);
    const campaignId = parseOptionalId(req.query.campaign_id);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([
        countCampaignTracks(db, { search, campaignId }),
        listCampaignTracks(db, { limit, offset, search, campaignId })
      ])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/campaign-tracks', asyncRoute(async (req, res) => {
    const r = validateCampaignTrack(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createCampaignTrack(db, r.value));
    res.status(201).json({ row });
  }));

  app.put('/api/campaign-tracks/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const r = validateCampaignTrack(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => updateCampaignTrack(db, id, r.value));
    if (!row) { res.status(404).json({ error: 'Campaign track not found.' }); return; }
    res.json({ row });
  }));

  app.delete('/api/campaign-tracks/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) { res.status(400).json({ error: 'Invalid id.' }); return; }
    const deleted = await withDatabase(getDatabaseConfig(), (db) => deleteCampaignTrack(db, id));
    if (!deleted) { res.status(404).json({ error: 'Campaign track not found.' }); return; }
    res.status(204).send();
  }));
}
