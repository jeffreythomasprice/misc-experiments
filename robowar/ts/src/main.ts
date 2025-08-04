import "./style.css";
import rapider2d from "@dimforge/rapier2d";

const canvas = document.createElement("canvas");
const context = canvas.getContext("2d");
if (!context) {
	throw new Error("Failed to get canvas context");
}
document.body.replaceChildren(canvas);

const resize = () => {
	const dpr = window.devicePixelRatio || 1;
	const displayWidth = window.innerWidth;
	const displayHeight = window.innerHeight;

	canvas.width = displayWidth * dpr;
	canvas.height = displayHeight * dpr;
};
resize();
window.onresize = resize;

const physicsWorldSize: rapider2d.Vector2 = {
	x: 100,
	y: 100,
};
const physicsWorld = new rapider2d.World({ x: 0, y: 9.81 });

// how much time is one physics step
physicsWorld.timestep = 1 / 60;

physicsWorld.createCollider(rapider2d.ColliderDesc.polyline(
	new Float32Array([
		0, 0,
		physicsWorldSize.x, 0,
		physicsWorldSize.x, physicsWorldSize.y,
		0, physicsWorldSize.y,
	]),
	new Uint32Array([
		0, 1,
		1, 2,
		2, 3,
		3, 0,
	])
));

const ballRigidBody = physicsWorld.createRigidBody(rapider2d.RigidBodyDesc.dynamic()
	.setTranslation(physicsWorldSize.x / 2, physicsWorldSize.y / 2));
const ballRadius = 5;
physicsWorld.createCollider(
	rapider2d.ColliderDesc.ball(ballRadius)
		.setRestitution(0.7),
	ballRigidBody
);

// the last time as given to us by requestAnimationFrame
let lastTime: number | null = null;
// the time in the physics simulation, may be larger than the last tick time
let lastPhysicsTime = 0;
// the time in the physics simulation corresponding to the exact last tick
let lastPhysicsTickTime = 0;
const render = (time: number) => {
	context.resetTransform();
	context.fillStyle = "white";
	context.fillRect(0, 0, canvas.width, canvas.height);

	const displayWidth = window.innerWidth;
	const displayHeight = window.innerHeight;
	const scale = Math.min(displayWidth / physicsWorldSize.x, displayHeight / physicsWorldSize.y);
	const transform = new DOMMatrix().scale(scale, scale);
	context.setTransform(transform);

	context.lineWidth = 5 / scale;
	context.strokeStyle = "black";

	context.beginPath();
	context.ellipse(
		ballRigidBody.translation().x,
		ballRigidBody.translation().y,
		ballRadius,
		ballRadius,
		0,
		0,
		Math.PI * 2
	);
	context.stroke();

	if (typeof lastTime === "number") {
		// in seconds
		const delta = (time - lastTime) / 1000;

		const newPhysicsTime = lastPhysicsTickTime + delta;
		const ticksToDo = Math.floor((newPhysicsTime - lastPhysicsTickTime) / physicsWorld.timestep);
		for (let i = 0; i < ticksToDo; i++) {
			lastPhysicsTickTime += physicsWorld.timestep;
			physicsWorld.step();
		}
		lastPhysicsTime = newPhysicsTime;
	}
	lastTime = time;

	requestAnimationFrame(render);
};
requestAnimationFrame(render);
