#pragma once

#include "common.h"

Napi::Promise execInNewThread(const Napi::Env& env,
							  std::function<Napi::Value(const Napi::Env&)> f);

// if value is a promise registers a then handler
// otherwise returns the value in the callback immediately
void promiseThen(const Napi::Env& env, Napi::Value value,
				 std::function<void(Napi::Value)> callback);

Napi::Value await(
	const Napi::ThreadSafeFunction& f,
	std::function<Napi::Value(const Napi::Env& env, Napi::Function f)>
		callback);
