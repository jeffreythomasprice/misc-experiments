```
cd frontend
gleam run -m lustre/dev start
```

```
cd backend
watchexec -r --stop-signal KILL gleam run
```

```
curl localhost:8001/counter \
	-v | jq

curl localhost:8001/counter \
	-X PUT \
	-H "Content-Type: application/json" \
	-d '{
		"increment_by": 1
	}' \
	-v | jq
```