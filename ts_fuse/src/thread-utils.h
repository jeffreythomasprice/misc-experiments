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
	std::binary_semaphore s(0);
	T result;
	f.BlockingCall(
		(void*)nullptr,
		[invokeThreadSafeFunctionCallback, castResultToNonJSValueCallback, &result, &s](const Napi::Env& env, Napi::Function f, void*) {
			promiseThen(env, invokeThreadSafeFunctionCallback(env, f), [castResultToNonJSValueCallback, &result, &s](const Napi::Value& r) {
				result = castResultToNonJSValueCallback(r);
				s.release();
			});
		}
	);
	s.acquire();
	return result;
}

void await(
	const Napi::ThreadSafeFunction& f, std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback
);

// TODO some helpers that take initializer lists instead of callbacks for
// invokeThreadSafeFunctionCallback