// ── Sectors ──────────────────────────────────────────────────────────────────

export async function listSectors(db) {
  const { rows } = await db.query('SELECT id, name FROM sectors ORDER BY name');
  return rows;
}

// ── Advertisers ───────────────────────────────────────────────────────────────

const ADV_SEL = `
  a.id, a.name, a.sector_id, a.address, a.vat_number, a.notes, a.active,
  TO_CHAR(a.client_since, 'YYYY-MM-DD') AS client_since,
  s.name AS sector_name
`;
const ADV_FROM = `FROM advertisers a LEFT JOIN sectors s ON s.id = a.sector_id`;

export async function listAdvertisers(db) {
  const { rows } = await db.query(`SELECT ${ADV_SEL} ${ADV_FROM} ORDER BY a.name`);
  return rows;
}

export async function getAdvertiser(db, id) {
  const { rows } = await db.query(`SELECT ${ADV_SEL} ${ADV_FROM} WHERE a.id = $1`, [id]);
  return rows[0] || null;
}

export async function createAdvertiser(db, data) {
  const { rows } = await db.query(
    `INSERT INTO advertisers (name, sector_id, address, vat_number, notes, active, client_since)
     VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id`,
    [data.name, data.sector_id, data.address, data.vat_number, data.notes, data.active, data.client_since]
  );
  return getAdvertiser(db, rows[0].id);
}

export async function updateAdvertiser(db, id, data) {
  await db.query(
    `UPDATE advertisers
     SET name=$2, sector_id=$3, address=$4, vat_number=$5, notes=$6, active=$7, client_since=$8
     WHERE id=$1`,
    [id, data.name, data.sector_id, data.address, data.vat_number, data.notes, data.active, data.client_since]
  );
  return getAdvertiser(db, id);
}

export async function deleteAdvertiser(db, id) {
  const { rowCount } = await db.query('DELETE FROM advertisers WHERE id=$1', [id]);
  return rowCount > 0;
}

// ── Contacts ──────────────────────────────────────────────────────────────────

const CON_SEL = `
  c.id, c.advertiser_id, c.name, c.role, c.phone, c.email, c.primary_contact, c.notes,
  a.name AS advertiser_name
`;
const CON_FROM = `FROM contacts c LEFT JOIN advertisers a ON a.id = c.advertiser_id`;

export async function listContacts(db) {
  const { rows } = await db.query(`SELECT ${CON_SEL} ${CON_FROM} ORDER BY a.name, c.name`);
  return rows;
}

export async function getContact(db, id) {
  const { rows } = await db.query(`SELECT ${CON_SEL} ${CON_FROM} WHERE c.id = $1`, [id]);
  return rows[0] || null;
}

export async function createContact(db, data) {
  const { rows } = await db.query(
    `INSERT INTO contacts (advertiser_id, name, role, phone, email, primary_contact, notes)
     VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id`,
    [data.advertiser_id, data.name, data.role, data.phone, data.email, data.primary_contact, data.notes]
  );
  return getContact(db, rows[0].id);
}

export async function updateContact(db, id, data) {
  await db.query(
    `UPDATE contacts
     SET advertiser_id=$2, name=$3, role=$4, phone=$5, email=$6, primary_contact=$7, notes=$8
     WHERE id=$1`,
    [id, data.advertiser_id, data.name, data.role, data.phone, data.email, data.primary_contact, data.notes]
  );
  return getContact(db, id);
}

export async function deleteContact(db, id) {
  const { rowCount } = await db.query('DELETE FROM contacts WHERE id=$1', [id]);
  return rowCount > 0;
}

// ── Campaigns ─────────────────────────────────────────────────────────────────

const CMP_SEL = `
  cp.id, cp.advertiser_id, cp.name, cp.station_id, cp.active,
  cp.total_broadcasts, cp.broadcast_count,
  TO_CHAR(cp.start_date, 'YYYY-MM-DD') AS start_date,
  TO_CHAR(cp.end_date,   'YYYY-MM-DD') AS end_date,
  a.name AS advertiser_name,
  s.name AS station_name
`;
const CMP_FROM = `
  FROM campaigns cp
  LEFT JOIN advertisers a ON a.id = cp.advertiser_id
  LEFT JOIN stations    s ON s.id = cp.station_id
`;

export async function listCampaigns(db) {
  const { rows } = await db.query(`SELECT ${CMP_SEL} ${CMP_FROM} ORDER BY a.name, cp.name`);
  return rows;
}

export async function getCampaign(db, id) {
  const { rows } = await db.query(`SELECT ${CMP_SEL} ${CMP_FROM} WHERE cp.id = $1`, [id]);
  return rows[0] || null;
}

export async function createCampaign(db, data) {
  const { rows } = await db.query(
    `INSERT INTO campaigns (advertiser_id, name, station_id, total_broadcasts, active, start_date, end_date)
     VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id`,
    [data.advertiser_id, data.name, data.station_id, data.total_broadcasts, data.active, data.start_date, data.end_date]
  );
  return getCampaign(db, rows[0].id);
}

export async function updateCampaign(db, id, data) {
  await db.query(
    `UPDATE campaigns
     SET advertiser_id=$2, name=$3, station_id=$4, total_broadcasts=$5, active=$6, start_date=$7, end_date=$8
     WHERE id=$1`,
    [id, data.advertiser_id, data.name, data.station_id, data.total_broadcasts, data.active, data.start_date, data.end_date]
  );
  return getCampaign(db, id);
}

export async function deleteCampaign(db, id) {
  const { rowCount } = await db.query('DELETE FROM campaigns WHERE id=$1', [id]);
  return rowCount > 0;
}

// ── Campaign Tracks ───────────────────────────────────────────────────────────

const CT_SEL = `
  ct.id, ct.campaign_id, ct.track_id, ct.position,
  cp.name AS campaign_name,
  ar.name || ' — ' || t.title AS track_display,
  t.title AS track_title, ar.name AS artist_name
`;
const CT_FROM = `
  FROM campaign_tracks ct
  LEFT JOIN campaigns cp ON cp.id = ct.campaign_id
  LEFT JOIN tracks t     ON t.id  = ct.track_id
  LEFT JOIN artists ar   ON ar.id = t.artist_id
`;

export async function listCampaignTracks(db) {
  const { rows } = await db.query(`SELECT ${CT_SEL} ${CT_FROM} ORDER BY cp.name, ct.position`);
  return rows;
}

export async function getCampaignTrack(db, id) {
  const { rows } = await db.query(`SELECT ${CT_SEL} ${CT_FROM} WHERE ct.id = $1`, [id]);
  return rows[0] || null;
}

async function nextPosition(db, campaignId) {
  const { rows } = await db.query(
    'SELECT COALESCE(MAX(position), 0) + 1 AS next FROM campaign_tracks WHERE campaign_id = $1',
    [campaignId]
  );
  return rows[0].next;
}

export async function createCampaignTrack(db, data) {
  const position = data.position > 0 ? data.position : (await nextPosition(db, data.campaign_id));
  const { rows } = await db.query(
    'INSERT INTO campaign_tracks (campaign_id, track_id, position) VALUES ($1,$2,$3) RETURNING id',
    [data.campaign_id, data.track_id, position]
  );
  return getCampaignTrack(db, rows[0].id);
}

export async function updateCampaignTrack(db, id, data) {
  await db.query(
    'UPDATE campaign_tracks SET campaign_id=$2, track_id=$3, position=$4 WHERE id=$1',
    [id, data.campaign_id, data.track_id, data.position]
  );
  return getCampaignTrack(db, id);
}

export async function deleteCampaignTrack(db, id) {
  const { rowCount } = await db.query('DELETE FROM campaign_tracks WHERE id=$1', [id]);
  return rowCount > 0;
}

// ── Track options (for selects) ───────────────────────────────────────────────

export async function listTrackOptions(db) {
  const { rows } = await db.query(`
    SELECT t.id, ar.name || ' — ' || t.title AS label
    FROM tracks t
    LEFT JOIN artists ar ON ar.id = t.artist_id
    ORDER BY ar.name, t.title
  `);
  return rows;
}
