document.addEventListener("DOMContentLoaded", async () => {
	const url = "ws://" + window.location.host + "/_liveReload";

	const defaultDelay = 100;
	const maxDelay = 10000;

	let delay = defaultDelay;

	let serverTime;

	while (true) {
		await new Promise(resolve => {
			const ws = new WebSocket(url);
			ws.addEventListener("open", () => {
				delay = defaultDelay;
				ws.send("test");
			});

			ws.addEventListener("message", (message) => {
				const newTime = message.data;
				if (serverTime) {
					if (newTime !== serverTime) {
						window.location.reload(true);
					}
				} else {
					serverTime = newTime;
				}
			});

			ws.addEventListener("error", () => {
				delay = Math.min(delay * 2, maxDelay);
				ws.close();
			});

			ws.addEventListener("close", () => {
				resolve();
			});
		});

		await new Promise(resolve => setTimeout(resolve, delay));
	}
});