import pg from 'pg';

export async function initDb({
	client,
}: {
	client: pg.Client;
}) {
	await client.query(`
		CREATE EXTENSION IF NOT EXISTS vector;

		CREATE TABLE IF NOT EXISTS document_chunk (
			id serial PRIMARY KEY,
			key text NOT NULL,
			first_page int NOT NULL,
			last_page int NOT NULL,
			-- TODO content should be a binary blob
			content text NOT NULL,
			-- text-embedding-3-small returns a vector of 1536 floats
			embedding vector(1536) NOT NULL,
			UNIQUE (key, first_page, last_page)
		);

		CREATE INDEX IF NOT EXISTS idx_document_chunk_embedding ON document_chunk USING hnsw (embedding vector_l2_ops);
	`);
}

export async function findAllByKey({
	client,
	key,
}: {
	client: pg.Client;
	key: string;
}): Promise<{
	id: number;
	key: string;
	first_page: number;
	last_page: number;
}[]> {

	const results = await client.query(
		"SELECT id, key, first_page, last_page FROM document_chunk WHERE key = $1",
		[
			key,
		],
	);
	return results.rows;
}

export async function insert({
	client,
	key,
	firstPage,
	lastPage,
	content,
	embedding,
}: {
	client: pg.Client;
	key: string;
	firstPage: number;
	lastPage: number;
	content: string;
	embedding: number[];
}) {
	await client.query(
		`INSERT INTO document_chunk (key, first_page, last_page, content, embedding) VALUES ($1, $2, $3, $4, $5)`,
		[
			key,
			firstPage,
			lastPage,
			content,
			JSON.stringify(embedding),
		],
	);
}

export async function search({
	client,
	keys,
	embedding,
	limit,
}: {
	client: pg.Client;
	keys: string[];
	embedding: number[];
	limit: number;
}): Promise<{
	id: number;
	key: string;
	first_page: number;
	last_page: number;
	content: string;
}[]> {
	const results = await client.query(
		"SELECT id, key, first_page, last_page, content FROM document_chunk WHERE key = ANY ($1) ORDER BY embedding <-> $2 LIMIT $3",
		[
			keys,
			JSON.stringify(embedding),
			limit,
		],
	);
	return results.rows;
}