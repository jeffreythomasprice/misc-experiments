#pragma once

#include "common.h"

Napi::Promise execInNewThread(const Napi::Env& env,
							  std::function<Napi::Value(const Napi::Env&)> f);
