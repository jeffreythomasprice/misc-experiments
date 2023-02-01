#pragma once

#include <memory>

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
