import { reset } from 'drizzle-seed';
// Adjust path to your db connection and schema
import { db } from '@/db';
import * as schema from '@/db/schema/index.ts'

async function clearDatabase() {
  console.log('🗑️ Clearing database...');
  await reset(db, schema);
  console.log('✅ Database cleared.');
  // Optional: close the database connection
  // await pg.end();
}

clearDatabase();
