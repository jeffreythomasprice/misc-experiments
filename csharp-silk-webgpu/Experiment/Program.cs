using var app = new App(
	(window) => new Experiment.WebGPU.VideoDriver(window),
	new AppStateTransition((windowState) => new Demo(windowState))
);
