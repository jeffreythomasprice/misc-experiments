#pragma once

#include <thread>

#include "common.h"

Napi::Promise execInNewThread(const Napi::Env& env, std::function<Napi::Value(const Napi::Env&)> f);

// if value is a promise registers a then handler
// otherwise returns the value in the callback immediately
void promiseThen(const Napi::Env& env, Napi::Value value, std::function<void(const Napi::Value&)> callback);

template <class T>
T await(
	const Napi::ThreadSafeFunction& f,
	std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback,
	std::function<T(const Napi::Value&)> castResultToNonJSValueCallback
) {
	// TODO check to make sure we're not on the js thread, don't block if we are
	std::binary_semaphore s(0);
	T result;
	f.BlockingCall(
		(void*)nullptr,
		[invokeThreadSafeFunctionCallback, castResultToNonJSValueCallback, &result, &s](const Napi::Env& env, Napi::Function f, void*) {
			printf("TODO JEFF in thread safe func\n");
			auto maybePromise = invokeThreadSafeFunctionCallback(env, f);
			auto thenCallback = [castResultToNonJSValueCallback, &result, &s](const Napi::Value& r) {
				printf("TODO JEFF got resuilt\n");
				result = castResultToNonJSValueCallback(r);
				printf("TODO JEFF got result converted, about to release\n");
				s.release();
			};
			printf("TODO JEFF about to attach promise.then callback\n");
			promiseThen(env, invokeThreadSafeFunctionCallback(env, f), thenCallback);
		}
	);
	printf("TODO JEFF about to wait on acquire\n");
	s.acquire();
	printf("TODO JEFF acquired\n");
	return result;
}

void await(
	const Napi::ThreadSafeFunction& f, std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback
);
