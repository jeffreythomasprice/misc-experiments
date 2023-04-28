#pragma once

#include <thread>

#include "common.h"

void initThreadUtils(const Napi::Env& env);

bool isNodeThread();

Napi::Promise execInNewThread(
	const Napi::Env& env, std::function<void()> onNewThreadCallback, std::function<Napi::Value(const Napi::Env&)> onNodeThreadCallback
);

// if value is a promise registers a then handler
// otherwise returns the value in the callback immediately
void promiseThen(const Napi::Env& env, Napi::Value value, std::function<void(const Napi::Value&)> callback);

void await(
	const Napi::ThreadSafeFunction& f, std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback
);
template <class T>
T await(
	const Napi::ThreadSafeFunction& f,
	std::function<Napi::Value(const Napi::Env& env, Napi::Function f)> invokeThreadSafeFunctionCallback,
	std::function<T(const Napi::Value&)> castResultToNonJSValueCallback
) {
	if (isNodeThread()) {
		throw std::logic_error("must execute await from a thread other than the node thread, will block waiting on a promise");
	}
	std::binary_semaphore s(0);
	T result;
	f.BlockingCall(
		(void*)nullptr,
		[invokeThreadSafeFunctionCallback, castResultToNonJSValueCallback, &result, &s](const Napi::Env& env, Napi::Function f, void*) {
			auto maybePromise = invokeThreadSafeFunctionCallback(env, f);
			auto thenCallback = [castResultToNonJSValueCallback, &result, &s](const Napi::Value& r) {
				result = castResultToNonJSValueCallback(r);
				s.release();
			};
			promiseThen(env, maybePromise, thenCallback);
		}
	);
	s.acquire();
	return result;
}
