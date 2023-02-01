#include "App.h"
#include "AppState.h"

#include <string>

#ifdef __EMSCRIPTEN__
#include <emscripten/val.h>
#endif

App::App(std::shared_ptr<AppState> initialState) : currentState(initialState)
{
	auto document = emscripten::val::global("document");
	auto body = document["body"];

	auto canvas = document.call<emscripten::val>("createElement", std::string("canvas"));
	canvas.set("id", "canvas");
	canvas["style"].set("position", "absolute");
	canvas["style"].set("width", "100%");
	canvas["style"].set("height", "100%");
	canvas["style"].set("left", "0");
	canvas["style"].set("top", "0");
	body.call<void>("replaceChildren", canvas);

	EmscriptenWebGLContextAttributes contextAttributes;
	emscripten_webgl_init_context_attributes(&contextAttributes);
	contextAttributes.powerPreference = EM_WEBGL_POWER_PREFERENCE_HIGH_PERFORMANCE;
	context = emscripten_webgl_create_context(canvas["id"].as<std::string>().c_str(), &contextAttributes);
	emscripten_webgl_make_context_current(context);

	currentState->activate();

	resize();

	auto x = emscripten_set_resize_callback(
		EMSCRIPTEN_EVENT_TARGET_WINDOW,
		// user data
		this,
		// use capture
		false,
		emscriptenWindowResize);

	emscripten_set_main_loop_arg(
		emscriptenMainLoop,
		// user data
		this,
		// desired FPS, 0 means requestAnimationFrame
		0,
		// simulate infinite loop, i.e. never return
		true);
}

App::~App()
{
	emscripten_html5_remove_all_event_listeners();
	emscripten_cancel_main_loop();
	emscripten_webgl_destroy_context(context);
}

void App::resize()
{
	auto window = emscripten::val::global("window");
	currentState->resize(window["innerWidth"].as<int>(), window["innerHeight"].as<int>());
}

void App::renderAndUpdate()
{
	emscripten_webgl_make_context_current(context);
	currentState->render();

	// TODO use real duration
	auto now = std::chrono::system_clock::now();
	if (lastUpdate.has_value())
	{
		auto newState = currentState->update(std::chrono::duration_cast<std::chrono::milliseconds>(now - lastUpdate.value()));
		if (newState && newState != currentState)
		{
			currentState->deactivate();
			newState->activate();
			currentState = newState;
			resize();
		}
	}
	lastUpdate = now;
}

EM_BOOL App::emscriptenWindowResize(int eventType, const EmscriptenUiEvent *uiEvent, void *userData)
{
	auto app = (App *)userData;
	app->resize();
	return EM_FALSE;
}

void App::emscriptenMainLoop(void *userData)
{
	auto app = (App *)userData;
	app->renderAndUpdate();
}