using var app = new App(new AppStateTransition(async (gl) =>
{
	return new Demo(gl);
}));
