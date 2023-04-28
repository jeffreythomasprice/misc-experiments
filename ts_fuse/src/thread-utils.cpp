#include "thread-utils.h"

#include <thread>

#include "logging.h"

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
		1);

	std::thread([done = std::move(done)]() {
		done.BlockingCall((void*)nullptr, [](const Napi::Env&, Napi::Function f,
											 void*) { f({}); });
		done.Release();
	}).detach();

	return deferred.Promise();
}

void promiseThen(const Napi::Env& env, Napi::Value value,
				 std::function<void(Napi::Value)> callback) {
	if (value.IsPromise()) {
		auto then = value.As<Napi::Promise>().Get("then");
		if (then.IsFunction()) {
			then.As<Napi::Function>().Call(
				value, {Napi::Function::New(
						   env, [callback](const Napi::CallbackInfo& info) {
							   callback(info[0]);
						   })});
			// TODO should handle error case too
		} else {
			warn() << "promise is not thenable";
			callback(value);
		}
	} else {
		callback(value);
	}
}

Napi::Value await(
	const Napi::ThreadSafeFunction& f,
	std::function<Napi::Value(const Napi::Env& env, Napi::Function f)>
		callback) {
	std::binary_semaphore s(0);
	Napi::Value result;
	f.BlockingCall(
		(void*)nullptr,
		[callback, &result, &s](const Napi::Env& env, Napi::Function f, void*) {
			promiseThen(env, callback(env, f), [&result, &s](Napi::Value r) {
				result = r;
				s.release();
				// TODO JEFF non-determinstically produces the wrong result,
				// meaning memory is broken here?
				debug() << "TODO JEFF result = " << result;
			});
		});
	s.acquire();
	return result;
}