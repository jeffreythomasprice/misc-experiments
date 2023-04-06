export interface MessageListEntry {
	id: string;
	timestamp: Date;
	channel: string;
	message: string;
}

export interface MessageListProps {
	messages: MessageListEntry[];
}

export function MessageList(props: MessageListProps) {
	return <ul>
		{props.messages.map(({ id, timestamp, channel, message }) =>
			<li key={id}>{`${timestamp.toISOString()} - ${channel} - ${message}`}</li>
		)}
	</ul>;
}
