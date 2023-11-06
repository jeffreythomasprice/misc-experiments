import { useEffect, useState } from "react";

interface HelloWorldResponse {
	message: string;
}

export function App() {
	let [message, setMessage] = useState<string | null>(null);

	useEffect(() => {
		(async () => {
			const start = Date.now();
			const response = await fetch("http://localhost:8001");
			console.log(`status = ${response.status}`);
			const responseBody: HelloWorldResponse = await response.json();
			const duration = Date.now() - start;
			console.log(`response = ${JSON.stringify(responseBody)}`);
			setMessage(responseBody.message);
			console.log(`duration = ${duration}`);
		})();

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

	if (message) {
		return <div>{message}</div>;
	}

	return <div>Loading...</div>;
}