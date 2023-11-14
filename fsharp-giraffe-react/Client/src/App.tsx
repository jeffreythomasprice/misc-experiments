import { useEffect, useState } from "react";

export function App() {
	useEffect(() => {
		// TODO make a typed websocket thing with json parsing?
		(async () => {
			const ws = new WebSocket("ws://127.0.0.1:8001/ws");
			ws.onopen = () => {
				console.log("ws opened");

				ws.send("sent from client");
			};
			ws.onclose = () => {
				console.log("ws closed");
			};
			ws.onerror = () => {
				console.log("ws error");
			};
			ws.onmessage = (event) => {
				if (typeof event.data === "string") {
					console.log(`ws message: ${event.data}`);
				} else {
					console.log("ws message unhandled type");
				}
			};
		})();
	}, []);

	return <Login></Login>;
}

interface LoginRequest {
	username: string;
	password: string;
}

function Login() {
	let [username, setUsername] = useState("");
	let [password, setPassword] = useState("");

	const submit = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();

		const request: LoginRequest = { username, password };

		// TODO helper service for interacting with server
		(async () => {
			const response = await fetch(
				"http://localhost:8001/login",
				{
					method: "POST",
					body: JSON.stringify(request),
				}
			);
			if (response.status < 200 || response.status >= 300) {
				throw new Error(`request failed, status code: ${response.status}`);
			}
		})()
			.catch(err => {
				console.error("login failed", err);
			});
	};

	return <form
		onSubmit={submit}
	>
		<div>
			<label htmlFor="username">Username: </label>
			<input
				type="text"
				placeholder="Username"
				name="username"
				value={username}
				onChange={(e) => setUsername(e.target.value)}
			></input>
		</div>
		<div>
			<label htmlFor="password">Password: </label>
			<input
				type="password"
				placeholder="Password"
				name="password"
				value={password}
				onChange={(e) => setPassword(e.target.value)}
			></input>
		</div>
		<button type="submit">Log In</button>
	</form>;
}