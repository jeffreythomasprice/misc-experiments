```
rm -rf server/.build \
	server/Sources/generated/*.swift \
	client/dist \
	client/src/generated/*.ts
```

```
cd client
bunx quicktype --out src/generated/generated.ts --src-lang schema --lang typescript ../schemas/*.json
bun run build --watch
```

```
cd server
bunx quicktype --out Sources/generated/generated.swift --src-lang schema --lang swift ../schemas/*.json
watchexec -r swift run
```