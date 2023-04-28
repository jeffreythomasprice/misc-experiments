#include "thread-utils.h"

#include "logging.h"

std::optional<std::thread::id> nodeThreadId;

void initThreadUtils(const Napi::Env& env) {
	nodeThreadId = std::this_thread::get_id();
}

bool isNodeThread() {
	if (!nodeThreadId.has_value()) {
		throw std::logic_error("thread utils not initialized");
	}
	return std::this_thread::get_id() == nodeThreadId.value();
}

Napi::Promise execInNewThread(
	const Napi::Env& env, std::function<void()> onNewThreadCallback, std::function<Napi::Value(const Napi::Env&)> onNodeThreadCallback
) {
	auto deferred = Napi::Promise::Deferred::New(env);

	auto done = Napi::ThreadSafeFunction::New(
		env,
		Napi::Function::New(
			env,
			[deferred, onNodeThreadCallback](const Napi::CallbackInfo& info) {
				auto env = info.Env();
				auto result = onNodeThreadCallback(env);
				deferred.Resolve(result);
			}
		),
		"deferred exec in new thread",
		// max queue size, 0 = unlimited
		0,
		// initial thread count
		1
	);

	std::thread([onNewThreadCallback, done = std::move(done)]() {
		onNewThreadCallback();
		done.BlockingCall((void*)nullptr, [](const Napi::Env&, Napi::Function f, void*) {
			f({});
		});
		done.Release();
	}).detach();

	return deferred.Promise();
}

void promiseThen(const Napi::Env& env, Napi::Value value, std::function<void(const Napi::Value&)> callback) {
	if (value.IsPromise()) {
		auto then = value.As<Napi::Promise>().Get("then");
		if (then.IsFunction()) {
			auto thenCallback = Napi::Function::New(env, [callback](const Napi::CallbackInfo& info) {
				callback(info[0]);
			});
			then.As<Napi::Function>().Call(value, {thenCallback});
			// TODO should handle error case too
		} else {
			warn() << "promise is not thenable";
			callback(value);
		}
	} else {
		callback(value);
	}
}

void await(
	const Napi::ThreadSafeFunction& f, std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback
) {
	await<int>(f, invokeThreadSafeFunctionCallback, [](const Napi::Value&) {
		return 0;
	});
}