import { Subject } from "rxjs";
import { ClientToServerMessage, ServerToClientMessage } from "./models";

export class WebsocketService {
	readonly messages = new Subject<ServerToClientMessage>();

	private ws: WebSocket | null = null;
	private shouldBeRunning = false;
	private isOpen = false;

	constructor(private readonly url: string) { }

	start(id: string) {
		if (this.ws) {
			this.stop();
		}

		this.shouldBeRunning = true;

		this.ws = new WebSocket(this.url);

		this.ws.addEventListener("open", () => {
			console.log("websocket opened");
			this.isOpen = true;

			const message: ClientToServerMessage = {
				type: "login",
				id,
			};
			this.ws?.send(JSON.stringify(message));
		});

		this.ws.addEventListener("close", () => {
			console.log("websocket closed");
			this.isOpen = false;
			if (this.shouldBeRunning) {
				this.start(id);
			}
		});

		this.ws.addEventListener("error", () => {
			console.error("error in websocket, restarting");
			this.ws?.close();
		});

		this.ws.addEventListener("message", async (message) => {
			try {
				const data = message.data;
				if (typeof data === "string") {
					this.handleTextMessage(data);
				} else if (data instanceof ArrayBuffer) {
					await this.handleArrayBufferMessage(data);
				} else if (data instanceof Blob) {
					await this.handleArrayBufferMessage(await data.arrayBuffer());
				} else {
					console.error("unhandled message type", typeof data, data);
				}
			} catch (err) {
				console.error("error handling websocket message", err);
			}
		});
	}

	stop() {
		if (!this.ws) {
			return;
		}
		this.ws.close();
		this.ws = null;
	}

	send(message: string) {
		const wrappedMessage: ClientToServerMessage = {
			type: "send",
			message,
		};
		if (this.isOpen) {
			this.ws?.send(JSON.stringify(wrappedMessage));
		} else {
			console.error("can't send message, websocket not ready yet");
		}
	}

	private handleTextMessage(data: string) {
		this.messages.next(JSON.parse(data));
	}

	private async handleArrayBufferMessage(data: ArrayBuffer) {
		this.handleTextMessage(new TextDecoder("utf-8").decode(data));
	}
}