import { Client } from 'pg';

export async function withDatabase(databaseConfig, callback) {
  const client = new Client(databaseConfig);
  await client.connect();

  try {
    return await callback(client);
  } finally {
    await client.end();
  }
}
