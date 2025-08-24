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

Ok. It looks like we cannot update a message. Which is fine. 

- [ ] Send messages to a specific user
- [ ] Send messages to a specific topic

## Notes

### Understanding Kafka/Redpanda as a Log File

Kafka and Redpanda can be thought of as **distributed log files** rather than traditional databases. This fundamental concept explains many of their behaviors:

#### **Log File Analogy**

Think of a topic as a **log file** where:

- Each line is a **message** with a sequential **offset** (line number)
- You can only **append** new lines at the end
- You can **never modify** or **delete** existing lines
- The file is **immutable** - once written, it's permanent

#### **What You CAN Do**

✅ **Append new messages** - Like adding lines to a log file  
✅ **Read from any position** - Start reading from any offset  
✅ **Read sequentially** - Process messages in order  
✅ **Send tombstones** - Mark messages as "deleted" (but they still exist)  
✅ **Use message metadata** - Timestamps, keys, etc.  

#### **What You CANNOT Do**

❌ **Modify existing messages** - Lines in a log file are immutable  
❌ **Delete messages** - You can't remove lines from a log file  
❌ **Reorder messages** - The sequence is fixed by offset  
❌ **Update in place** - No "UPDATE" operations like in databases  

#### **Practical Implications**

**For Chat Applications:**

- "Updating" a message = sending a new message with `replaces_offset`
- "Deleting" a message = sending a new message with `delete_offset`
- Clients must handle the logic of showing/hiding messages based on these metadata

This is a bad design IMHO. I'll probably add a 'verb' field to the message to make it more clear.

**For Data Processing:**

- Perfect for event streaming and real-time analytics
- Excellent for audit trails and data lineage
- Great for decoupling systems (pub/sub pattern)

#### **Why This Design?**

This immutable log design provides:

- **High performance** - Sequential writes are very fast
- **Durability** - Data is never lost once written
- **Scalability** - Easy to partition and distribute
- **Replay capability** - Can replay events from any point in time
- **Event sourcing** - Complete history of all events
