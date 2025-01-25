Carton can't figure out that some versions of ubuntu are compatible with each other. Manually installing swift wasm versions for carton:
- Download a release from https://github.com/swiftwasm/swift/releases
	- e.g. `swift-wasm-6.1-SNAPSHOT-2025-01-17-a-ubuntu22.04_x86_64.tar.gz`
- Edit swift version to have that name without the leading `swift-` or the trailing os specifier
	- e.g. `wasm-6.1-SNAPSHOT-2025-01-17-a`
- Put the extracted `.tar.gz` contents in `~/.carton/sdk/<version>`
	- e.g. `/home/jeff/.carton/sdk/wasm-6.1-SNAPSHOT-2025-01-17-a/usr/`

```
make watch
```
