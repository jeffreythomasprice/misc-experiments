import { useEffect, useState } from "react";

import { API_BASE_URL } from "../env";
import { APIClient } from "../../../shared";
import { MessageList, MessageListEntry } from "./MessageList";
import { WebSocketClient } from "../services/WebSocketClient";
import { ChannelService } from "../services/ChannelService";
import { ChannelList } from "./ChannelList";
import { ChannelSelector } from "./ChannelSelector";

export function App() {
	let [isInit, setIsInit] = useState(false);
	let [apiClient, setApiClient] = useState<APIClient | null>(null);
	let [channelService, setChannelService] = useState<ChannelService | null>(null);
	let [_websocketClient, setWebsocketClient] = useState<WebSocketClient | null>(null);

	let [messages, setMessages] = useState<MessageListEntry[]>([]);
	let [allChannels, setAllChannels] = useState<string[]>([]);
	let [currentChannels, setCurrentChannels] = useState<string[]>([]);
	let [activeChannel, setActiveChannel] = useState<string | null>(null);

	let [messageInputValue, setMessageInputValue] = useState("");
	let [channelInputValue, setChannelInputValue] = useState("");

	useEffect(
		() => {
			if (isInit) {
				return;
			}
			setIsInit(true);

			const apiClient = new APIClient(API_BASE_URL);
			setApiClient(apiClient);

			const channelService = new ChannelService(apiClient);
			setChannelService(channelService);
			channelService.allChannelsObservable.subscribe((channels) => {
				channels.sort((a, b) => a.localeCompare(b));
				setAllChannels(channels);
			});
			channelService.currentChannelsObservable.subscribe((channels) => {
				channels.sort((a, b) => a.localeCompare(b));
				setCurrentChannels(channels);
			});

			const websocketClient = new WebSocketClient(apiClient);
			setWebsocketClient(websocketClient);

			// initial login, other services should be watching the login status and kick off their data loads as needed
			void apiClient.login();

			websocketClient.observable.subscribe((message) => {
				messages.push({
					id: message.id,
					timestamp: new Date(message.timestamp),
					channel: message.channel,
					message: message.message
				});
				setMessages([...messages]);
			});

			return () => {
				websocketClient?.close();
				setWebsocketClient(null);
			};
		},
		[]
	);

	const sendMessage = async () => {
		try {
			if (activeChannel === null) {
				return;
			}
			await apiClient?.sendMessage({
				channel: activeChannel,
				message: messageInputValue
			});
		} catch (e) {
			console.error("error sending message", e);
		}
	};

	const joinChannel = async () => {
		try {
			await channelService?.join(channelInputValue);
		} catch (e) {
			console.error("error joining channel", e);
		}
		if (activeChannel === null) {
			setActiveChannel(channelInputValue);
		}
		setChannelInputValue("");
	};

	return <>
		<MessageList messages={messages}></MessageList>

		<form onSubmit={(e) => {
			e.preventDefault();
			void sendMessage();
		}}>
			<input type="submit" style={{ display: "none" }}></input>
			<label>
				Message
				<input id="message" type="text" onChange={(e) => {
					setMessageInputValue(e.target.value);
				}}></input>
			</label>
			<ChannelSelector all={allChannels} current={currentChannels} onChange={(active) => {
				setActiveChannel(active);
			}}></ChannelSelector>
			<button type="button" disabled={activeChannel === null} onClick={() => {
				void sendMessage();
			}}>Send</button>
		</form>

		<form onSubmit={(e) => {
			e.preventDefault();
			void joinChannel();
		}}>
			<label>
				Channel
				<input id="message" type="text" value={channelInputValue} onChange={(e) => {
					setChannelInputValue(e.target.value);
				}}></input>
			</label>
			<button type="button" onClick={() => {
				void joinChannel();
			}}>Join Channel</button>
		</form>

		<ChannelList channels={currentChannels} onLeave={(channel) => {
			void channelService?.leave(channel);
		}}></ChannelList>
	</>
}
