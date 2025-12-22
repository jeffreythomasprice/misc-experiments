import fsPromise from "fs/promises";
import OpenAI from "openai";
import type { EasyInputMessage } from "openai/resources/responses/responses.mjs";
import os from "os";
import path from "path";
import pg from 'pg';
import * as db from "./db";
import { assertEnvVar, assertIntEnvVar } from "./env";
import { extractPdfPagesIntoNewPdf, extractPdfText, getPdfPageCount } from "./pdf";

// TODO example is python pydantic-ai, but this is the openai embeddings thing
// https://ai.pydantic.dev/examples/rag/

// TODO proper logging

async function createEmbeddings({
	openAIClient,
	input,
}: {
	openAIClient: OpenAI;
	input: string;
}): Promise<number[]> {
	const embeddings = await openAIClient.embeddings.create({
		model: "text-embedding-3-small",
		input,
	});
	const result = embeddings.data[0]?.embedding;
	if (embeddings.data.length !== 1 || !result) {
		throw new Error("expected exactly one embeddings result");
	}
	return result;
}

async function createEmbeddingsFromPages({
	key,
	inputPath,
	firstPage,
	lastPage,
	tempDir,
	openAIClient,
}: {
	key: string;
	inputPath: string;
	firstPage: number;
	lastPage: number;
	tempDir: string;
	openAIClient: OpenAI;
}): Promise<{
	textContent: string;
	embeddings: number[];
}> {
	const chunkPath = await extractPdfPagesIntoNewPdf({
		inputPath: inputPath,
		firstPage,
		lastPage: lastPage,
		outputDir: tempDir,
	});
	const chunkTextContent = await extractPdfText({
		path: chunkPath,
	});
	const result = await createEmbeddings({
		openAIClient,
		input: [
			`key: ${key}`,
			`start page: ${firstPage}`,
			`end page: ${lastPage}`,
			`content: ${chunkTextContent}`,
		].join("\n\n"),
	});
	console.log(`created embeddings key=${key}, firstPage=${firstPage}, lastPage=${lastPage}`);
	return {
		textContent: chunkTextContent,
		embeddings: result,
	};
}

async function chunkPdf({
	openAIClient,
	pgClient,
	path: inputPath,
	tempDir,
	maxChunkPageCount,
}: {
	openAIClient: OpenAI;
	pgClient: pg.Client;
	path: string;
	tempDir: string;
	maxChunkPageCount: number;
}): Promise<{
	key: string;
}> {
	const key = inputPath;

	const pageCount = await getPdfPageCount({
		path: inputPath,
	});
	console.log(`${inputPath}, page count: ${pageCount}`);

	// skip if we already have all pages
	const existing = await db.findAllByKey({
		client: pgClient,
		key,
	});
	if (existing.length > 0) {
		const firstExistingPage = existing.map(x => x.first_page).reduce((a, b) => Math.min(a, b));
		const lastExistingPage = existing.map(x => x.last_page).reduce((a, b) => Math.max(a, b));
		if (firstExistingPage === 1 && lastExistingPage === pageCount) {
			console.log("existing page range appears to be covered in db, skipping embeddings");
			return {
				key,
			};
		}
	}

	let firstPage = 1;
	outerLoop: while (firstPage <= pageCount) {
		for (let chunkPageCount = maxChunkPageCount; chunkPageCount >= 1; chunkPageCount--) {
			const lastPage = Math.min(firstPage + chunkPageCount, pageCount);

			// skip if this exact page range already exists
			if (existing.find(e => e.first_page === firstPage && e.last_page === lastPage)) {
				console.log(`skipping because found existing page for ${firstPage}:${lastPage}`);
			} else {
				try {
					const embeddings = await createEmbeddingsFromPages({
						key,
						inputPath,
						firstPage,
						lastPage,
						tempDir,
						openAIClient,
					});

					await db.insert({
						client: pgClient,
						key,
						firstPage,
						lastPage,
						content: embeddings.textContent,
						embedding: embeddings.embeddings,
					});
				} catch (e) {
					if (!!e && e instanceof OpenAI.APIError) {
						// if it's trying to tell us we have provided too many tokens, guess how many pages would be a good number of tokens and try again
						const m = /This model's maximum context length is ([0-9]+) tokens, however you requested ([0-9]+) tokens/.exec(e.message);
						if (m) {
							const [, maxTokensStr, inputTokensStr] = m;
							const maxTokens = parseInt(maxTokensStr!, 10);
							const inputTokens = parseInt(inputTokensStr!, 10);
							const averageTokensPerPage = inputTokens / chunkPageCount;
							const estimatedNumberOfPages = Math.max(Math.floor(maxTokens / averageTokensPerPage), 1);
							chunkPageCount = Math.min(chunkPageCount, estimatedNumberOfPages + 1);
							continue;
						} else {
							throw e;
						}
					} else {
						throw e;
					}
				}

				firstPage = Math.max(firstPage + 1, lastPage - 1);
				continue outerLoop;
			}
		}
	}

	return {
		key,
	};
}

async function searchEmbeddings({
	openAIClient,
	pgClient,
	keys,
	query,
}: {
	openAIClient: OpenAI;
	pgClient: pg.Client;
	keys: string[];
	query: string;
}): Promise<string[]> {
	const inputEmbeddings = await createEmbeddings({
		openAIClient,
		input: query,
	});
	const results = await db.search({
		client: pgClient,
		keys,
		embedding: inputEmbeddings,
		limit: 10,
	});
	return results.map(x => `
		This content comes from ${x.key} (page ${x.first_page}):
		-----
		${x.content}
		-----
	`);
}

// TODO name of script? name of project in package.json?
const tempDir = await fsPromise.mkdtemp(path.join(os.tmpdir(), "experiment"));
try {
	console.log(`temp dir: ${tempDir}`);

	const pgClient = new pg.Client({
		host: assertEnvVar("PG_HOST"),
		port: assertIntEnvVar("PG_PORT"),
		user: assertEnvVar("PG_USERNAME"),
		password: assertEnvVar("PG_PASSWORD"),
		database: assertEnvVar("PG_DATABASE"),
	});
	try {
		await pgClient.connect();
		await db.initDb({
			client: pgClient,
		});

		const openAIClient = new OpenAI();

		const {
			key: pdfKey,
		} = await chunkPdf({
			openAIClient,
			pgClient,
			path: "/home/jeff/scratch/games/source_material/free_or_stolen/World of Darkness (Classic)/v20 Vampire The Masquerade - 20th Anniversary Edition.pdf",
			tempDir,
			// TODO what should the max chunk size be?
			maxChunkPageCount: 25,
		});

		const relevantPdfPages = await searchEmbeddings({
			openAIClient,
			pgClient,
			keys: [
				pdfKey,
			],
			query: "initiative combat rules",
		});

		const response = await openAIClient.responses.create({
			model: "gpt-5-nano",
			// model: "gpt-5-mini",
			input: [
				{
					type: "message",
					role: "system",
					content: `
					You're an assistant searching through documents for answers to questions.

					Prefer concise answers. You can answer in sentence fragments if the whole meaning is still clear.

					If the question asks for a list, you can answer in bullet point lists.
					`,
				},
				...relevantPdfPages.map((content): EasyInputMessage => {
					return {
						type: "message",
						role: "assistant",
						content,
					};
				}),
				{
					type: "message",
					role: "user",
					content: "How does initiative work?",
					// content: "My character has that 4th dot blood sorcery power that drains blood at range. How does this work in combat? What do I roll?",
					// content: "The party got some claymore mines. What sort of stats do they have?",
				},
			],
		});
		console.log(response.output_text);
	} finally {
		try {
			await pgClient.end();
		} catch (e) {
			console.error("error stopping postgres client", e);
		}
	}
} finally {
	try {
		console.log(`cleaning up temp dir: ${tempDir}`);
		await fsPromise.rm(
			tempDir,
			{
				recursive: true,
				force: true,
			}
		);
	} catch (e) {
		console.error(`error cleaning up temp dir: ${tempDir}`, e);
	}
}
