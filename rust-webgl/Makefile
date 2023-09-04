.PHONY: \
	watch-cube \
	lint \
	lint-fix

watch-cube:
	cd demo-cube && trunk serve --watch ../lib --watch .

lint:
	cargo clippy

lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged
	cargo fmt