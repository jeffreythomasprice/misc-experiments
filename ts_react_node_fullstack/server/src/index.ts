import fs from "fs/promises";

import Koa from "koa";
import KoaRouter from "koa-router";
import KoaBodyParser from "koa-bodyparser";
import KoaWebsocket, { MiddlewareContext as KoaWebsocketContext } from "koa-websocket";
import { WebSocket } from "ws";
import * as uuid from "uuid";
import jsonwebtoken from "jsonwebtoken";

import { GetChannelsResponse, JoinChannelRequest, LeaveChannelRequest, LoginResponse, SendMessageRequest, WebsocketMessage } from "../../shared";
import { KafkaService } from "./kafka";

class HTTPError extends Error {
	constructor(public readonly statusCode: number, message?: string) {
		super(message);
	}
}

interface Connection {
	id: string;
	ws: WebSocket | null;
	expireTimeout: NodeJS.Timeout | null;
	connected: boolean;
	channels: Set<string>;
}

interface JwtPayload extends jsonwebtoken.JwtPayload {
	id: string;
}

(async () => {
	const kafka = new KafkaService({
		clientId: "experiment",
		brokers: [
			"localhost:9092"
		]
	});

	const serverId = uuid.v4();
	console.log(`server starting ${serverId}`);

	const connections = new Map<string, Connection>();

	const addChannel = async (channel: string) => {
		await kafka.addTopic<WebsocketMessage>(
			getTopicForChannel(channel),
			(_payload, data) => {
				for (const connection of connections.values()) {
					if (!connection.ws) {
						continue;
					}
					if (connection.channels.has(data.channel)) {
						sendToWebsocket(connection.ws, data);
					}
				}
				return Promise.resolve();
			}
		);
	};

	const removeChannel = async (channel: string) => {
		await kafka.removeTopic(getTopicForChannel(channel));
	};

	const jwtPrivateKey = await fs.readFile("../certs/jwt-private.pem");
	const jwtPublicKey = await fs.readFile("../certs/jwt-public.pem");

	const app = KoaWebsocket(new Koa());

	app.use(KoaBodyParser());

	app.use((ctx, next) => {
		ctx.set("Access-Control-Allow-Origin", "*");
		ctx.set("Access-Control-Allow-Headers", "*");
		ctx.set("Access-Control-Allow-Methods", "GET, OPTIONS, POST, PUT, PATCH, DELETE");
		return next();
	});

	app.use(async (ctx, next) => {
		try {
			await next();
		} catch (e) {
			if (e instanceof HTTPError) {
				console.log(`http error, status code: ${e.statusCode}`);
				ctx.status = e.statusCode;
			} else {
				console.error("error", e);
				ctx.status = 500;
			}
		}
	});

	useRoutesAndAllowedMethods(
		app,
		new KoaRouter()
			.post("/login", (ctx) => {
				const id = uuid.v4();
				console.log(`login ${id}`);

				const connection: Connection = {
					id,
					ws: null,
					expireTimeout: setTimeout(
						() => {
							console.log(`${id} never connected`);
							connections.delete(id);
						},
						5000
					),
					connected: false,
					channels: new Set()
				};
				connections.set(id, connection);

				const response: LoginResponse = {
					id,
					token: createJwt(jwtPrivateKey, {
						id,
						exp: new Date().valueOf() + 1000 * 60 * 15
					})
				};
				ctx.status = 200;
				ctx.body = response;
			})
			.post("/send", async (ctx) => {
				const senderId = verifyJwt(jwtPublicKey, ctx).id;
				const request = ctx.request.body as SendMessageRequest;
				const message: WebsocketMessage = {
					id: uuid.v4(),
					senderId,
					timestamp: new Date().valueOf(),
					channel: request.channel,
					message: request.message
				};
				console.log(`sending message ${JSON.stringify(message)}`);
				await addChannel(message.channel);
				await kafka.send({
					topic: getTopicForChannel(message.channel),
					messages: [
						{
							value: JSON.stringify(message)
						}
					]
				});
				ctx.status = 200;
			})
	);

	useRoutesAndAllowedMethods(
		app,
		new KoaRouter()
			.prefix("/channel")
			.get("/all", async (ctx) => {
				verifyJwt(jwtPublicKey, ctx).id;
				const topics = await kafka.listTopics();
				const response: GetChannelsResponse = {
					channels: topics
						// TODO helper functions down with the other one
						.filter((topic) => topic.startsWith("channel-"))
						.map((topic) => topic.substring("channel-".length))
				};
				ctx.status = 200;
				ctx.body = response;
			})
			.get("/current", (ctx) => {
				const senderId = verifyJwt(jwtPublicKey, ctx).id;
				const connection = connections.get(senderId);
				if (!connection) {
					throw new HTTPError(404, "no such connection");
				}
				const response: GetChannelsResponse = {
					channels: [...connection.channels]
				};
				ctx.status = 200;
				ctx.body = response;
			})
			.post("/join", async (ctx) => {
				const senderId = verifyJwt(jwtPublicKey, ctx).id;
				const connection = connections.get(senderId);
				if (!connection) {
					throw new HTTPError(404, "no such connection");
				}
				const request = ctx.request.body as JoinChannelRequest;
				connection.channels.add(request.channel);
				await addChannel(request.channel);
				console.log(`added ${request.channel}, connection ${senderId} is now in these channels: ${JSON.stringify([...connection.channels])}`);
				ctx.status = 200;
			})
			.post("/leave", async (ctx) => {
				const senderId = verifyJwt(jwtPublicKey, ctx).id;
				const connection = connections.get(senderId);
				if (!connection) {
					throw new HTTPError(404, "no such connection");
				}
				const request = ctx.request.body as LeaveChannelRequest;
				connection.channels.delete(request.channel);
				await removeChannel(request.channel);
				console.log(`removed ${request.channel}, connection ${senderId} is now in these channels: ${JSON.stringify([...connection.channels])}`);
				ctx.status = 200;
			})
	);

	app.ws.use((ctx: KoaWebsocketContext<Koa.DefaultState>) => {
		const ws = ctx.websocket;
		try {
			const payload = verifyJwt(jwtPublicKey, ws.protocol);

			const connection = connections.get(payload.id);
			if (!connection) {
				console.log(`no such connection ${payload.id}`);
				ws.close();
				return;
			}
			if (connection.connected) {
				throw new Error(`duplicate connection, must login again ${payload.id}`);
			}

			console.log(`${payload.id} connected`);
			connection.connected = true;
			connection.ws = ctx.websocket;
			if (connection.expireTimeout) {
				clearTimeout(connection.expireTimeout);
				connection.expireTimeout = null;
			}

			ctx.websocket.on("close", (code, reason) => {
				console.log(`connection ${payload.id} closed, code=${code}, reason=${reason.toString()}`);
				connections.delete(payload.id);
			});

			ctx.websocket.on("error", (err) => {
				console.error(`connection websocket error ${payload.id} error`, err);
			});
		} catch (e) {
			console.error("error handling websocket connection", e);
			ws.close();
		}
	});

	const port = 8001;
	console.log(`${serverId} listening on ${port}`);
	app.listen(port);
})()
	.catch((e) => {
		console.error("fatal", e);
	});

function useRoutesAndAllowedMethods(target: KoaRouter | Koa, router: KoaRouter) {
	return target.use(router.routes()).use(router.allowedMethods());
}

function createJwt(privateKey: Buffer, payload: JwtPayload): string {
	return jsonwebtoken.sign(payload, privateKey, { algorithm: "RS256" });
}

function verifyJwt(publicKey: Buffer, token: string | undefined | null | Koa.ParameterizedContext): JwtPayload {
	// if we're in an http handler and have been given the whole context, try the auth header
	if (token && typeof token === "object") {
		token = token.headers.authorization;
	}
	if (!token) {
		console.log("no token");
		throw new HTTPError(401);
	}
	// accept an auth header with the Bearer prefix, but just keep the token as is if no such prefix
	const m = /^Bearer\s+(.+)\s*$/.exec(token);
	if (m) {
		token = m[1];
	}
	try {
		const jwt = jsonwebtoken.verify(token, publicKey, { complete: true });
		return jwt.payload as JwtPayload;
	} catch (e) {
		console.error("jwt verification failed", e);
		throw new HTTPError(401);
	}
}

function getTopicForChannel(channel: string): string {
	return `channel-${channel}`;
}

async function sendToWebsocket<T>(ws: WebSocket, data: T) {
	return new Promise<void>((resolve, reject) => {
		ws.send(JSON.stringify(data), (err) => {
			if (err) {
				reject(err);
			} else {
				resolve();
			}
		});
	});
}
