import { withDatabase } from '../db/client.js';
import {
  countUsers,
  createUser,
  deleteUser,
  getUser,
  listUsers,
  updateUser
} from '../repositories/users.js';

const LIMIT = 50;

function parsePagination(query) {
  const page  = Math.max(1, parseInt(query.page  || 1, 10) || 1);
  const limit = Math.min(200, Math.max(1, parseInt(query.limit || LIMIT, 10) || LIMIT));
  return { page, limit, offset: (page - 1) * limit };
}

function parseId(value) {
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : null;
}

function validateUser(data, { requirePassword = true } = {}) {
  const login = String(data?.login || '').trim();
  if (!login) {
    return { ok: false, error: 'Login is required.' };
  }

  if (login.length > 32) {
    return { ok: false, error: 'Login must be 32 characters or less.' };
  }

  const password = String(data?.password || '').trim();
  if (requirePassword && !password) {
    return { ok: false, error: 'Password is required.' };
  }

  const active = Boolean(data?.active);
  const role_id = parseInt(data?.role_id ?? 0, 10);
  if (!Number.isInteger(role_id) || role_id <= 0) {
    return { ok: false, error: 'Role is required.' };
  }

  return { ok: true, value: { login, password: password || null, active, role_id } };
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

export function registerUserRoutes(app, getDatabaseConfig) {
  app.get('/api/users', asyncRoute(async (req, res) => {
    const { page, limit, offset } = parsePagination(req.query);
    const [total, rows] = await withDatabase(getDatabaseConfig(), (db) =>
      Promise.all([countUsers(db), listUsers(db, { limit, offset })])
    );
    res.json({ rows, total, page, limit });
  }));

  app.get('/api/users/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid user id.' });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) => getUser(db, id));
    if (!row) {
      res.status(404).json({ error: 'User not found.' });
      return;
    }

    res.json({ row });
  }));

  app.post('/api/users', asyncRoute(async (req, res) => {
    const user = validateUser(req.body, { requirePassword: true });
    if (!user.ok) {
      res.status(400).json({ error: user.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      createUser(db, user.value)
    );
    res.status(201).json({ row });
  }));

  app.put('/api/users/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid user id.' });
      return;
    }

    const user = validateUser(req.body, { requirePassword: false });
    if (!user.ok) {
      res.status(400).json({ error: user.error });
      return;
    }

    const row = await withDatabase(getDatabaseConfig(), (db) =>
      updateUser(db, id, user.value)
    );
    if (!row) {
      res.status(404).json({ error: 'User not found.' });
      return;
    }

    res.json({ row });
  }));

  app.delete('/api/users/:id', asyncRoute(async (req, res) => {
    const id = parseId(req.params.id);
    if (!id) {
      res.status(400).json({ error: 'Invalid user id.' });
      return;
    }

    const deleted = await withDatabase(getDatabaseConfig(), (db) =>
      deleteUser(db, id)
    );
    if (!deleted) {
      res.status(404).json({ error: 'User not found.' });
      return;
    }

    res.status(204).send();
  }));
}
