import { APIClient, WebsocketMessage } from "../../../shared";
import { Observable, Subject } from "rxjs";
import { WS_BASE_URL } from "../env";

export class WebSocketClient {
	private websocket: WebSocket | undefined;
	private readonly subject = new Subject<WebsocketMessage>();
	private allowReopen = true;

	constructor(private readonly apiClient: APIClient) {
		// reconnect if the auth token changes because somebody else logged in
		this.apiClient.authTokenObservable.subscribe(() => {
			void this.connect();
		});
		// connect at startup if we can
		if (!this.apiClient.needsLogin) {
			void this.connect();
		}
	}

	get observable(): Observable<WebsocketMessage> {
		return this.subject;
	}

	close(allowReopen = false) {
		this.allowReopen = allowReopen;
		if (this.websocket) {
			this.websocket.close();
			this.websocket = undefined;
		}
		if (!allowReopen) {
			this.subject.complete();
		}
	}

	private async connect() {
		try {
			this.close(true);
			if (!this.allowReopen) {
				console.log("websocket closed forever");
				return;
			}

			if (this.apiClient.needsLogin) {
				console.error("can't connect to websocket, not logged in");
				return;
			}

			this.websocket = new WebSocket(WS_BASE_URL, this.apiClient.authToken);

			this.websocket.onopen = () => {
				console.log("websocket opened");
			};

			this.websocket.onerror = (e) => {
				console.error("error in websocket", e);
			};

			this.websocket.onmessage = (e) => {
				const messageStr = e.data?.toString();
				if (!messageStr) {
					return;
				}
				const message = JSON.parse(messageStr) as WebsocketMessage;
				this.subject.next(message);
			};

			this.websocket.onclose = async () => {
				console.log("websocket closed");
				// force a new login, which should trigger a reconnect by listening for the new token
				void this.apiClient.login();
			};
		} catch (e) {
			console.error("error connecting", e);
			setTimeout(
				() => {
					// force a new login, which should trigger a reconnect by listening for the new token
					void this.apiClient.login();
				},
				1000
			);
		}
	}
}
