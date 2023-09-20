import { type Component, createSignal, Show } from "solid-js";
import { Login } from "./Login";
import { Service } from "./Service";
import { LoggedIn } from "./LoggedIn";
import { WebsocketService } from "./WebsocketService";

interface LoggedInUser {
	name: string;
	id: string;
}

export const App: Component = () => {
	const service = new Service("http://localhost:8001");
	const websocketService = new WebsocketService("ws://localhost:8001/ws");

	const [user, setUser] = createSignal<LoggedInUser | null>(null);

	return <>
		<Show
			when={user()}
			fallback={
				<Login
					service={service}
					callback={(response) => {
						websocketService.start(response.id);
						setUser(response);
					}}
				/>
			}
		>
			{user =>
				<LoggedIn
					name={user().name}
					service={websocketService}
				/>}
		</Show>
	</>;
};
