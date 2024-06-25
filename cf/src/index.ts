/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.toml`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

import { Hono } from 'hono';
import { cors } from 'hono/cors';

const app = new Hono<{ Bindings: Env }>();

app.use('/api/*', cors());

app.get('/api/pet', async (c) => {
	return c.json({
		count: (await c.env.DB.prepare('SELECT count FROM pets').first())!.count,
	});
});

app.post('/api/pet', async (c) => {
	return c.json({
		count: (await c.env.DB.prepare('UPDATE pets SET count = count + 1 RETURNING count').first())!.count,
	});
});

export default app satisfies ExportedHandler<Env>;
