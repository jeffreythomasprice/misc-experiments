#include "thread-utils.h"

#include "logging.h"

Napi::Promise execInNewThread(const Napi::Env& env, std::function<Napi::Value(const Napi::Env&)> f) {
	auto deferred = Napi::Promise::Deferred::New(env);

	auto done = Napi::ThreadSafeFunction::New(
		env,
		Napi::Function::New(
			env,
			[deferred, f](const Napi::CallbackInfo& info) {
				auto env = info.Env();
				auto result = f(env);
				deferred.Resolve(result);
			}
		),
		"deferred exec in new thread",
		// max queue size, 0 = unlimited
		0,
		// initial thread count
		1
	);

	std::thread([done = std::move(done)]() {
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