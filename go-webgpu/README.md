```
go run .
```

```
mkdir -p bin
GOOS=js GOARCH=wasm go build -o bin/wasm .
cp "$(go env GOROOT)/lib/wasm/wasm_exec.js" bin/
npx http-server -p 8000
```