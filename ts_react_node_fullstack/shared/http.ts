import { Observable, Subject } from "rxjs";
import jsonwebtoken from "jsonwebtoken";

export interface LoginResponse {
	id: string;
	token: string;
}

export interface SendMessageRequest {
	channel: string;
	message: string;
}

export interface GetChannelsResponse {
	channels: string[];
}

export interface JoinChannelRequest {
	channel: string;
}

export interface LeaveChannelRequest {
	channel: string;
}

export class APIClient {
	private readonly baseUrl: string;

	private _authToken: string | undefined;
	private authTokenSubject = new Subject<string>();

	constructor(baseUrl: string) {
		if (baseUrl.endsWith("/")) {
			this.baseUrl = baseUrl.slice(0, baseUrl.length - 1);
		} else {
			this.baseUrl = baseUrl;
		}
	}

	get authToken() {
		return this._authToken;
	}

	get authTokenObservable(): Observable<string> {
		return this.authTokenSubject;
	}

	get authHeader() {
		if (this.authToken) {
			return `Bearer ${this.authToken}`;
		}
	}

	get needsLogin() {
		// missing token
		if (!this._authToken) {
			return true;
		}
		// token is an actual jwt?
		const result = jsonwebtoken.decode(this._authToken, { complete: true, json: true });
		if (!result) {
			return true;
		}
		// token is valid, does it have an expiration time?
		const exp = (result.payload as jsonwebtoken.JwtPayload).exp;
		if (!exp) {
			return true;
		}
		// expired
		if (exp < new Date().valueOf()) {
			return true;
		}
		// expiration time is in the future, we're good
		return false;
	}

	async login(): Promise<LoginResponse> {
		this._authToken = undefined;
		const result = await this.makeJsonRequest<LoginResponse>("/login", { method: "POST" });
		this._authToken = result.token;
		this.authTokenSubject.next(result.token);
		return result;
	}

	async sendMessage(request: SendMessageRequest): Promise<void> {
		await this.makeRequest("/send", makePostWithBody(request));
	}

	getAllChannels(): Promise<GetChannelsResponse> {
		return this.makeJsonRequest("/channel/all");
	}

	getCurrentChannels(): Promise<GetChannelsResponse> {
		return this.makeJsonRequest("/channel/current");
	}

	async joinChannel(channel: string): Promise<void> {
		const request: JoinChannelRequest = { channel };
		await this.makeRequest("/channel/join", makePostWithBody(request));
	}

	async leaveChannel(channel: string): Promise<void> {
		const request: JoinChannelRequest = { channel };
		await this.makeRequest("/channel/leave", makePostWithBody(request));
	}

	private async makeRequest(uri: string, requestInit?: RequestInit): Promise<Response> {
		if (uri.startsWith("/")) {
			uri = uri.slice(1);
		}

		if (this.authHeader) {
			if (this.needsLogin) {
				// TODO login again to refresh
			}
			requestInit = shallowCopyAndAddHeader(requestInit, "Authorization", this.authHeader);
		}

		const method = requestInit?.method ?? "GET";

		const response = await fetch(`${this.baseUrl}/${uri}`, requestInit);
		if (!response.ok) {
			throw new Error(`${method} ${response.url} failed ${response.statusText}`);
		}
		return response;
	}

	private async makeJsonRequest<T>(uri: string, requestInit?: RequestInit): Promise<T> {
		requestInit = shallowCopyAndAddHeader(requestInit, "Accept", "application/json");
		const response = await this.makeRequest(uri, requestInit);
		const responseBody = await response.json();
		return responseBody;
	}
}

function makePostWithBody(body: any): RequestInit {
	return {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify(body)
	};
}

function shallowCopyAndAddHeader(requestInit: RequestInit | undefined, header: string, value: string): RequestInit {
	return {
		...(requestInit ?? {}),
		headers: {
			...(requestInit?.headers ?? {}),
			[header]: value
		}
	};
}