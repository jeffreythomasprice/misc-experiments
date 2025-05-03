using var app = new App(
	(window) => new WebGPUVideoDriver(window),
	new AppStateTransition((windowState) => new Demo(windowState))
);
