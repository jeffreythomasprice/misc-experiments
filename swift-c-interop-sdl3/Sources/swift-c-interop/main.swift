import CLib
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

let window = SDL_CreateWindow("Experiment", 1024, 768, 0)
if window == nil {
	let error = String(cString: SDL_GetError())
	print("failed to create SDL window: \(error)")
	exit(1)
}
defer {
	SDL_DestroyWindow(window)
}

let renderer = SDL_CreateRenderer(window, nil)
defer {
	SDL_DestroyRenderer(renderer)
}

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

	SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255)
	SDL_RenderClear(renderer)
	SDL_RenderPresent(renderer)

	SDL_Delay(20)
}
