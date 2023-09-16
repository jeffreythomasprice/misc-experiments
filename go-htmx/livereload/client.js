document.addEventListener("DOMContentLoaded", async () => {
	const baseUrl = `ws://${window.location.host}`;
	const path = "{{.Path}}";
	let url;
	if (path.startsWith("/")) {
		url = `${baseUrl}${path}`;
	} else {
		url = `${baseUrl}/${path}`;
	}

	const defaultDelay = 100;
	const maxDelay = 10000;

	let delay = defaultDelay;

	let serverTime;

	while (true) {
		await new Promise(resolve => {
			// TODO sometimes new websockets hang
			const ws = new WebSocket(url);
			ws.addEventListener("open", () => {
				delay = defaultDelay;
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