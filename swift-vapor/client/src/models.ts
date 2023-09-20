export interface LoginRequest {
	name: string;
}

export interface LoginResponse {
	id: string;
}

export interface ClientToServerMessage {
	message: string;
}

export interface ServerToClientMessage {
	senderId: string;
	message: string;
}