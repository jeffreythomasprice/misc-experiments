#include <iostream>
#include <chrono>
#include <optional>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/val.h>
#include <emscripten/fetch.h>
#endif

#include <GL/gl.h>

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
	EMSCRIPTEN_WEBGL_CONTEXT_HANDLE context;

public:
	App(std::shared_ptr<AppState> initialState);
	~App();

	void resize();
	void renderAndUpdate();

	static EM_BOOL emscriptenWindowResize(int eventType, const EmscriptenUiEvent *uiEvent, void *userData);
	static void emscriptenMainLoop(void *userData);
};

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

void fetchOnSuccess(emscripten_fetch_t *fetch)
{
	std::cout << "TODO JEFF on success" << std::endl;
	emscripten_fetch_close(fetch);
}

void fetchOnError(emscripten_fetch_t *fetch)
{
	std::cout << "TODO JEFF on error" << std::endl;
	emscripten_fetch_close(fetch);
}

void fetchOnProgress(emscripten_fetch_t *fetch)
{
	std::cout << "TODO JEFF on progress, numBytes = " << fetch->numBytes << ", totalBytes = " << fetch->totalBytes << std::endl;
}

//   void (*onsuccess)(struct emscripten_fetch_t *fetch);
//   void (*onerror)(struct emscripten_fetch_t *fetch);
//   void (*onprogress)(struct emscripten_fetch_t *fetch);

std::optional<std::string> fetchString(const std::string &url, const std::optional<std::chrono::milliseconds> &timeout = std::nullopt)
{
	emscripten_fetch_attr_t attributes;
	emscripten_fetch_attr_init(&attributes);
	strcpy(attributes.requestMethod, "GET");
	attributes.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY;
	attributes.onsuccess = fetchOnSuccess;
	attributes.onerror = fetchOnError;
	attributes.onprogress = fetchOnProgress;
	if (timeout.has_value())
	{
		attributes.timeoutMSecs = timeout.value().count();
	}
	std::cout << "TODO JEFF calling emscripten_fetch" << std::endl;
	auto fetch = emscripten_fetch(&attributes, url.c_str());
	std::cout << "TODO JEFF emscripten_fetch returned, status = " << fetch->statusText << std::endl;
	// TODO JEFF actually read data
	return std::nullopt;
}

class DemoState : public AppState
{
public:
	void activate() override
	{
		auto result = fetchString("index.html");
		if (result.has_value())
		{
			std::cout << "TODO JEFF fetch result = " << result.value() << std::endl;
		}
		else
		{
			std::cout << "TODO JEFF fetch had no result" << std::endl;
		}
	}

	void resize(int width, int height) override
	{
		glViewport(0, 0, width, height);
	}

	void render() override
	{
		glClearColor(0.25f, 0.5f, 0.75f, 1.0f);
		glClear(GL_COLOR_BUFFER_BIT);
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