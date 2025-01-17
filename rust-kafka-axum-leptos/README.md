leptosfmt isn't working right, trigger manually
```
leptosfmt --max-width 140 client server shared
```

```
docker compose up -d
```

```
docker compose down -v
```

```
docker exec -it broker /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
docker exec -it broker /opt/kafka/bin/kafka-consumer-groups.sh --bootstrap-server localhost:9092 --describe --all-groups --all-topics
```

```
cd server
cargo watch -s "cargo run -- --config-file config.json"
```

```
cd client
trunk serve
```


TODO chat app

topics:
- messages_all
- messages_channel_x
	- x is a channel name

consumer groups:
- forward
	- all nodes are members
	- reads from messages_all
	- writes to messages_channel_x, where x is pulled from the message body
- db
	- all nodes are members
	- reads from messages_all
	- writes to db
- websockets
	- nodes become members as needed based on what channels their connected websocket clients care about
	- reads from messages_channel_x, for all x where a connected websocket cares about that channel
	- writes to all websockets that care about that channel

sets of nodes, nodes can be part of multiple sets:
- websockets
	- in the websocket consumer group
	- commands from the clients to send messages get written to messages_all
	- listening to messages_channel_x as part of the websockets consumer group, writing messages to those websockets
	- websockets can request a time range from the db, where it just looks those up directly and sends them in batches over the websocket
- forward
	- the forward consumer group
- db
	- the db consumer group
