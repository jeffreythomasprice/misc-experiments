export interface ChannelListProps {
	channels: string[];
	onLeave: (channel: string) => void;
}

export function ChannelList(props: ChannelListProps) {
	return <ul>
		{props.channels.map((channel) =>
			<li key={channel}>
				{channel}
				<button type="button" onClick={() => {
					props.onLeave(channel);
				}}>
					Leave
				</button>
			</li>
		)}
	</ul>;
}
