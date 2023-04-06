export interface WebsocketMessage {
	id: string;
	senderId: string;
	channel: string;
	timestamp: number;
	message: string;
}
