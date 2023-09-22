export interface LoginRequest {
	name: string;
}

export interface LoginResponse {
	id: string;
}

export namespace ClientToServerMessage {
	export interface Send {
		type: "send";
		message: string;
	}

	export interface Login {
		type: "login";
		id: string;
	}
}

export type ClientToServerMessage = ClientToServerMessage.Send | ClientToServerMessage.Login;

export namespace ServerToClientMessage {
	export interface Send {
		type: "send";
		senderId: string;
		message: string;
	}
}

export type ServerToClientMessage = ServerToClientMessage.Send;