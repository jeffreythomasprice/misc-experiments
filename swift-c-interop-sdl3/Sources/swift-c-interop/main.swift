import CLib
import COpenGL
import CSDL

print("add result = \(add(1,2))")

if !SDL_Init(SDL_INIT_VIDEO) {
	let error = String(cString: SDL_GetError())
	print("SDL init error: \(error)")
	exit(1)
}
defer {
	SDL_Quit()
}

print("SDL version = \(SDL_GetVersion())")

let SDL_WINDOW_OPENGL = SDL_WindowFlags(0x0000_0000_0000_0002)
let window = SDL_CreateWindow("Experiment", 1024, 768, SDL_WINDOW_OPENGL)
if window == nil {
	let error = String(cString: SDL_GetError())
	print("failed to create SDL window: \(error)")
	exit(1)
}
defer {
	SDL_DestroyWindow(window)
}

let glContext = SDL_GL_CreateContext(window)

let framesPerSecond = 60
let delayBetweenFrames = 1000 / framesPerSecond

var exiting = false
while !exiting {
	var e = SDL_Event.init()
	if SDL_PollEvent(&e) {
		switch SDL_EventType(rawValue: Int32(e.type)) {
		case SDL_EVENT_QUIT:
			exiting = true
			break
		case SDL_EVENT_KEY_UP:
			switch e.key.key {
			case SDLK_ESCAPE:
				exiting = true
				break
			default:
				break
			}
			break
		default:
			break
		}
	}

	glClearColor(0.25, 0.5, 1, 1)
	glClear(GLbitfield(GL_COLOR_BUFFER_BIT))
	SDL_GL_SwapWindow(window)

	SDL_Delay(Uint32(delayBetweenFrames))
}
