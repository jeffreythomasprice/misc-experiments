document.addEventListener("DOMContentLoaded", () => {
	main().catch(err => {
		console.error("oops", err);
	});
});

async function main() {
	const response = await fetch(
		"/test",
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				message: "hello from client"
			})
		}
	);
	console.log(`got response, status = ${response.status}, ${response.statusText}`);
	const responseBody = await response.text();
	console.log(`response body:\n${responseBody}`);

	const ws = new WebSocket(`ws://${document.location.host}/ws`);
	ws.addEventListener("open", () => {
		ws.send("hello from client");
	});
	ws.addEventListener("message", (message) => {
		console.log("received message", message.data);
	});
}