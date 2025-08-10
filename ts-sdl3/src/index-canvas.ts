import sdl from '@kmamal/sdl'
import { createCanvas } from '@napi-rs/canvas'

// Setup
const window = sdl.video.createWindow({ title: "Canvas2D" });
const { pixelWidth: width, pixelHeight: height } = window;
const canvas = createCanvas(width, height);
const ctx = canvas.getContext('2d');

// Clear screen to red
ctx.fillStyle = 'red';
ctx.fillRect(0, 0, width, height);

// Render to window
const buffer = Buffer.from(ctx.getImageData(0, 0, width, height).data);
window.render(width, height, width * 4, 'rgba32', buffer);
