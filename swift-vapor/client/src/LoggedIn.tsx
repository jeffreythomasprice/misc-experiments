import { Component, For, createSignal, onCleanup, onMount } from "solid-js";
import { WebsocketService } from "./WebsocketService";
import { Subscription } from "rxjs";
import { ServerToClientMessage } from "./models";

interface Props {
	name: string;
	service: WebsocketService;
}

export const LoggedIn: Component<Props> = ({ name, service }) => {
	const [messageToSend, setMessageToSend] = createSignal("");
	const [messages, setMessages] = createSignal<ServerToClientMessage.Send[]>([]);

	let messagesSubscription: Subscription | null = null;

	onMount(() => {
		try {
			messagesSubscription = service.messages.subscribe((message) => {
				setMessages((messages) => {
					return [...messages, message];
				});
			});
		} catch (err) {
			console.error("error subscribing", err);
		}
	});

	onCleanup(() => {
		messagesSubscription?.unsubscribe();
		messagesSubscription = null;
	});

	return <>
		<p>Name: {name}</p>
		<form onsubmit={(e) => {
			e.preventDefault();
			service.send(messageToSend());
			setMessageToSend("");
		}}>
			<input
				type="text"
				placeholder="Message"
				value={messageToSend()}
				autofocus
				oninput={(x) => {
					setMessageToSend(x.target.value);
				}}
			></input>
			<button type="submit">Submit</button>
		</form >
		<For each={messages()}>{
			(message) => {
				return <p>{message.senderId} - {message.message}</p>
			}
		}</For>
	</>;
};
