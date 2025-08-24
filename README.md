# Redpanda

## Install Redpanda

```bash
docker run -d \
  --pull=always \
  --name=redpanda-1 \
  --rm \
  -p 9092:9092 \
  -p 9644:9644 \
  docker.redpanda.com/redpandadata/redpanda:latest \
  redpanda start \
  --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:9092 \
  --advertise-kafka-addr internal://redpanda-1:9092,external://localhost:9092 \
  --overprovisioned \
  --smp 1 \
  --memory 1G \
  --mode dev-container \
  --default-log-level=warn \
  --reserve-memory 0M
```

## Configure Redpanda

```bash
docker exec -it redpanda-1 rpk topic create chat-room --brokers localhost:9092
```

## Check cluster status

```bash
docker exec -it redpanda-1 rpk cluster health
```

## Check messages in the topic

```bash
docker exec -it redpanda-1 rpk topic consume chat-room --brokers localhost:9092
```

## Check messages in the topic

```bash
docker exec -it redpanda-1 rpk topic consume chat-room --brokers localhost:9092 
```

## Cleanup topic:

```bash
docker exec -it redpanda-1 rpk topic delete chat-room
docker exec -it redpanda-1 rpk topic create chat-room
```

## Example : 

### We start a first client (alice)

```bash
red-panda-test % cargo r -- -u alice
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/red-panda-test -u alice`
Bob: Hello
Salut !
alice: Salut !
```

### We start a second client (bob)

```bash
red-panda-test % cargo r -- -u Bob
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/red-panda-test -u Bob`
Hello
Bob: Hello
alice: Salut !
```

### We check the messages in the topic

```bash	
red-panda-test % docker exec -it redpanda-1 rpk topic consume chat-room --brokers localhost:9092
...
{
  "topic": "chat-room",
  "key": "key",
  "value": "Bob: Hello",
  "timestamp": 1756050582305,
  "partition": 0,
  "offset": 8
}
{
  "topic": "chat-room",
  "key": "key",
  "value": "alice: Salut !",
  "timestamp": 1756050586930,
  "partition": 0,
  "offset": 9
}
```

## TODO :

- [X] Send JSON / Protobuf messages

```bash
red-panda-test % cargo r -- -u alice -f json
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/red-panda-test -u alice -f json`
coucou
alice: coucou
```

```bash
red-panda-test % docker exec -it redpanda-1 rpk topic consume chat-room --brokers localhost:9092
{
  "topic": "chat-room",
  "key": "key",
  "value": "{\"username\":\"alice\",\"message\":\"coucou\"}",
  "timestamp": 1756053930760,
  "partition": 0,
  "offset": 12
}
```

- [ ] Update message content
- [ ] Send messages to a specific user
- [ ] Send messages to a specific topic
