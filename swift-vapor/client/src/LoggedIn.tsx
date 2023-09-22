import { Component, createSignal, onCleanup, onMount } from "solid-js";
import { WebsocketService } from "./WebsocketService";
import { Subscription } from "rxjs";

interface Props {
	name: string;
	service: WebsocketService;
}

export const LoggedIn: Component<Props> = ({ name, service }) => {
	const [message, setMessage] = createSignal("");

	let messagesSubscription: Subscription | null = null;

	onMount(() => {
		messagesSubscription = service.messages.subscribe((message) => {
			// TODO JEFF handle incoming message
			console.log("TODO JEFF incoming message", message);
		});
	});

	onCleanup(() => {
		messagesSubscription?.unsubscribe();
		messagesSubscription = null;
	});

	return <>
		<p>Name: {name}</p>
		<form onsubmit={(e) => {
			e.preventDefault();
			service.send(message());
			setMessage("");
		}}>
			<input
				type="text"
				placeholder="Message"
				value={message()}
				autofocus
				oninput={(x) => {
					setMessage(x.target.value);
				}}
			></input>
			<button type="submit">Submit</button>
		</form >
	</>;
};
