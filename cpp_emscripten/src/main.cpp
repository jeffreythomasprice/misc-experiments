#include <iostream>
#include <chrono>
#include <optional>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/val.h>
#endif

class AppState : public std::enable_shared_from_this<AppState>
{
public:
	virtual void activate() {}
	virtual void deactivate() {}
	virtual void resize(int width, int height) {}
	virtual void render() {}
	virtual std::shared_ptr<AppState> update(const std::chrono::milliseconds &d)
	{
		return shared_from_this();
	}
};

class App
{
private:
	std::shared_ptr<AppState> currentState;
	std::optional<std::chrono::system_clock::time_point> lastUpdate;

public:
	App(std::shared_ptr<AppState> initialState);

	void resize();
	void renderAndUpdate();

	static EM_BOOL emscriptenWindowResize(int eventType, const EmscriptenUiEvent *uiEvent, void *userData);
	static void emscriptenMainLoop(void *userData);
};

App::App(std::shared_ptr<AppState> initialState) : currentState(initialState)
{
	currentState->activate();

	resize();

	emscripten_set_resize_callback(
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

void App::resize()
{
	auto window = emscripten::val::global("window");
	currentState->resize(window["innerWidth"].as<int>(), window["innerHeight"].as<int>());
}

void App::renderAndUpdate()
{
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

class DemoState : public AppState
{
public:
	void resize(int width, int height) override
	{
		// std::cout << "DemoState resize " << width << " x " << height << std::endl;
	}

	void render() override
	{
		// std::cout << "DemoState render" << std::endl;
	}

	std::shared_ptr<AppState> update(const std::chrono::milliseconds &d) override
	{
		// std::cout << "DemoState update " << d.count() << "ms" << std::endl;
		return nullptr;
	}
};

int main()
{
	App app(std::make_shared<DemoState>());
	return 0;
}