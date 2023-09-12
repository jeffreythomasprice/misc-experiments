.PHONY: \
	client \
	server \
	lint \
	lint-fix

client:
	cd client && trunk serve --watch ../shared --watch .

server:
	cd server && cargo watch -x run

lint:
	cargo clippy

lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged
	cargo fmt