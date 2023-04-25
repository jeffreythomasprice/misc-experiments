#include "thread-utils.h"

#include <thread>

Napi::Promise execInNewThread(const Napi::Env& env,
							  std::function<Napi::Value(const Napi::Env&)> f) {
	auto deferred = Napi::Promise::Deferred::New(env);

	auto done = Napi::ThreadSafeFunction::New(
		env,
		Napi::Function::New(env,
							[deferred, f](const Napi::CallbackInfo& info) {
								auto env = info.Env();
								auto result = f(env);
								deferred.Resolve(result);
							}),
		"deferred exec in new thread",
		// max queue size, 0 = unlimited
		0,
		// initial thread count
		1,
		// finalizer
		[](Napi::Env) {});

	std::thread([done = std::move(done)]() {
		done.BlockingCall((void*)nullptr,
						  [](Napi::Env, Napi::Function f, void*) { f({}); });
		done.Release();
	}).detach();

	return deferred.Promise();
}