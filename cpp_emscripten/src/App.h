#pragma once

#include <memory>
#include <optional>

#ifdef __EMSCRIPTEN__
#include <emscripten/html5.h>
#endif

class AppState;

class App
{
private:
	std::shared_ptr<AppState> currentState;
	std::optional<std::chrono::system_clock::time_point> lastUpdate;
	EMSCRIPTEN_WEBGL_CONTEXT_HANDLE context;

public:
	App(std::shared_ptr<AppState> initialState);
	~App();

	void resize();
	void renderAndUpdate();

	static EM_BOOL emscriptenWindowResize(int eventType, const EmscriptenUiEvent *uiEvent, void *userData);
	static void emscriptenMainLoop(void *userData);
};