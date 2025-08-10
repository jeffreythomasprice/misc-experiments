import sdl from '@kmamal/sdl'
import createContext from '@kmamal/gl'

// Setup
const window = sdl.video.createWindow({ title: "WebGL", opengl: true });
const { pixelWidth: width, pixelHeight: height, native } = window;
const gl = createContext(width, height, { window: native });
if (!gl) {
	throw new Error("Failed to create WebGL context");
}

// Clear screen to red
gl.clearColor(1, 0, 0, 1);
gl.clear(gl.COLOR_BUFFER_BIT);

// Render to window
gl.swap();