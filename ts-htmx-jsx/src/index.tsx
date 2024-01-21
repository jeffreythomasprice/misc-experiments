import fastifyFormBody from "@fastify/formbody";
import fastify from "fastify";
import pino from "pino";
import React from "react";
import { htmlPageFunc, htmlSnippetFunc } from "./html-rendering";

(async () => {
	const log = pino({ level: "trace" });

	const server = fastify({
		logger: log,
	});
	server.register(fastifyFormBody);

	let clicks = 0;

	const clicksDiv = () => <div id="clicks">Clicks: {clicks}</div>;

	server.get("/", htmlPageFunc(() => <>
		{clicksDiv()}
		<button
			hx-post="/clicks"
			hx-target="#clicks"
		>Click Me</button>
	</>));

	server.post("/clicks", htmlSnippetFunc(() => {
		clicks++;
		return clicksDiv();
	}));

	await server.listen({
		host: "127.0.0.1",
		port: 8000,
	});
})()
	.catch(err => {
		console.error("fatal", err);
	});
