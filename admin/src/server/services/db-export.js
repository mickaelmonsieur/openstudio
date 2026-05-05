import zlib from 'node:zlib';
import { withDatabase } from '../db/client.js';

function sqlLiteral(value) {
  if (value === null || value === undefined) return 'NULL';
  if (typeof value === 'boolean') return value ? 'TRUE' : 'FALSE';
  if (typeof value === 'number') return String(value);
  if (value instanceof Date) return `'${value.toISOString().replace('T', ' ').replace('Z', '+00')}'`;
  if (Buffer.isBuffer(value)) return `'\\x${value.toString('hex')}'`;
  if (Array.isArray(value)) return `'{${value.map((v) => String(v).replace(/"/g, '\\"')).join(',')}}'`;
  return `'${String(value).replace(/'/g, "''")}'`;
}

async function getTableOrder(db) {
  // Topological sort: tables with no FK dependencies first
  const { rows: allTables } = await db.query(`
    SELECT table_name FROM information_schema.tables
    WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
    ORDER BY table_name
  `);

  const { rows: fkEdges } = await db.query(`
    SELECT tc.table_name AS from_table, ccu.table_name AS to_table
    FROM information_schema.table_constraints tc
    JOIN information_schema.referential_constraints rc
      ON tc.constraint_name = rc.constraint_name AND tc.constraint_schema = rc.constraint_schema
    JOIN information_schema.constraint_column_usage ccu
      ON rc.unique_constraint_name = ccu.constraint_name AND rc.unique_constraint_schema = ccu.constraint_schema
    WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = 'public'
      AND tc.table_name <> ccu.table_name
  `);

  const deps = new Map(allTables.map((t) => [t.table_name, new Set()]));
  for (const { from_table, to_table } of fkEdges) {
    if (deps.has(from_table)) deps.get(from_table).add(to_table);
  }

  const sorted = [];
  const visited = new Set();

  function visit(name) {
    if (visited.has(name)) return;
    visited.add(name);
    for (const dep of deps.get(name) || []) visit(dep);
    sorted.push(name);
  }

  for (const { table_name } of allTables) visit(table_name);
  return sorted;
}

export async function streamDatabaseExport(databaseConfig, res) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const filename = `openstudio-${timestamp}.sql.gz`;

  res.setHeader('Content-Type', 'application/gzip');
  res.setHeader('Content-Disposition', `attachment; filename="${filename}"`);

  const gzip = zlib.createGzip({ level: 9 });
  gzip.pipe(res);

  const write = (str) => gzip.write(str);

  try {
    await withDatabase(databaseConfig, async (db) => {
      write(`-- OpenStudio Database Export\n`);
      write(`-- Generated: ${new Date().toISOString()}\n`);
      write(`-- Format: plain SQL, gzip compressed\n\n`);
      write(`SET client_encoding = 'UTF8';\n`);
      write(`SET standard_conforming_strings = on;\n`);
      write(`SET session_replication_role = replica;\n\n`);

      const tables = await getTableOrder(db);

      for (const table of tables) {
        const { rows: cols } = await db.query(`
          SELECT column_name, data_type, udt_name
          FROM information_schema.columns
          WHERE table_schema = 'public' AND table_name = $1
          ORDER BY ordinal_position
        `, [table]);

        if (cols.length === 0) continue;

        const colList = cols.map((c) => `"${c.column_name}"`).join(', ');
        const { rows } = await db.query(`SELECT * FROM "${table}" ORDER BY 1`);

        write(`-- Table: ${table} (${rows.length} rows)\n`);
        write(`TRUNCATE TABLE "${table}" RESTART IDENTITY CASCADE;\n`);

        for (const row of rows) {
          const values = cols.map((c) => sqlLiteral(row[c.column_name])).join(', ');
          write(`INSERT INTO "${table}" (${colList}) VALUES (${values});\n`);
        }

        // Reset sequence if table has a serial/bigserial id column
        const hasSerial = cols.some(
          (c) => c.column_name === 'id' && ['integer', 'bigint'].includes(c.data_type)
        );
        if (hasSerial && rows.length > 0) {
          write(`SELECT setval(pg_get_serial_sequence('"${table}"', 'id'), MAX(id)) FROM "${table}";\n`);
        }

        write('\n');
      }

      write(`SET session_replication_role = DEFAULT;\n`);
    });
  } finally {
    gzip.end();
  }
}
