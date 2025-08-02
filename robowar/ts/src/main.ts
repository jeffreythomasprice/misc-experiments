import "./style.css";

const canvas = document.createElement("canvas");
const context = canvas.getContext("2d");
if (!context) {
	throw new Error("Failed to get canvas context");
}

document.body.replaceChildren(canvas);

window.onresize = () => {
	canvas.width = window.innerWidth;
	canvas.height = window.innerHeight;
};

const render = () => {
	const gradient = context.createLinearGradient(
		0,
		0,
		canvas.width,
		canvas.height,
	);
	gradient.addColorStop(0, "red");
	gradient.addColorStop(0.5, "green");
	gradient.addColorStop(1, "blue");
	context.fillStyle = gradient;

	context.fillRect(0, 0, canvas.width, canvas.height);

	requestAnimationFrame(render);
};
requestAnimationFrame(render);
