import { withDatabase } from '../db/client.js';
import {
  listSectors,
  countAdvertisers, listAdvertisers, getAdvertiser, createAdvertiser, updateAdvertiser, deleteAdvertiser,
  countContacts, listContacts, getContact, createContact, updateContact, deleteContact,
  countCampaigns, listCampaigns, getCampaign, createCampaign, updateCampaign, deleteCampaign,
  countCampaignTracks, listCampaignTracks, getCampaignTrack, createCampaignTrack, updateCampaignTrack, deleteCampaignTrack
} from '../repositories/advertising.js';

const ADV_LIMIT = 50;

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
  return {
    ok: true,
    value: {
      name,
      sector_id:   data?.sector_id ? parseId(data.sector_id) : null,
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
  return {
    ok: true,
    value: {
      advertiser_id,
      name:             str(data?.name, 255),
      station_id:       data?.station_id ? parseId(data.station_id) : null,
      total_broadcasts: Math.max(0, parseInt(data?.total_broadcasts || 0, 10) || 0),
      active:           Boolean(data?.active ?? true),
      start_date:       optDate(data?.start_date),
      end_date:         optDate(data?.end_date)
    }
  };
}

function validateCampaignTrack(data) {
  const campaign_id = parseId(data?.campaign_id);
  if (!campaign_id) return { ok: false, error: 'Campaign is required.' };
  const track_id = parseId(data?.track_id);
  if (!track_id) return { ok: false, error: 'Track is required.' };
  const pos = parseInt(data?.position || 0, 10);
  return {
    ok: true,
    value: { campaign_id, track_id, position: pos > 0 ? pos : null }
  };
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
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countCampaigns(db, search), listCampaigns(db, { limit, offset, search })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.post('/api/campaigns', asyncRoute(async (req, res) => {
    const r = validateCampaign(req.body);
    if (!r.ok) { res.status(400).json({ error: r.error }); return; }
    const row = await withDatabase(getDatabaseConfig(), (db) => createCampaign(db, r.value));
    res.status(201).json({ row });
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
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countCampaignTracks(db, search), listCampaignTracks(db, { limit, offset, search })])
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
