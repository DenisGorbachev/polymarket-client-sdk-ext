# Real Time Data Socket

## Overview

The Polymarket Real-Time Data Socket (RTDS) is a WebSocket-based streaming service that provides real-time updates for **comments** and **crypto prices**.

<Card title="TypeScript client" icon="github" href="https://github.com/Polymarket/real-time-data-client">
  Official RTDS TypeScript client (`real-time-data-client`).
</Card>

### Connection Details

* **WebSocket URL**: `wss://ws-live-data.polymarket.com`
* **Protocol**: WebSocket
* **Data Format**: JSON

### Authentication

Some user-specific streams may require `gamma_auth`:

* `address`: User wallet address

### Connection Management

The WebSocket connection supports:

* **Dynamic Subscriptions**: Without disconnecting from the socket users can add, remove and modify topics and filters they are subscribed to.
* **Ping/Pong**: You should send PING messages (every 5 seconds ideally) to maintain connection

## Available Subscription Types

<Note>Only the subscription types documented below are supported.</Note>

The RTDS currently supports the following subscription types:

1. **[Crypto Prices](/developers/RTDS/RTDS-crypto-prices)** - Real-time cryptocurrency price updates
2. **[Comments](/developers/RTDS/RTDS-comments)** - Comment-related events including reactions

## Message Structure

All messages received from the WebSocket follow this structure:

```json  theme={null}
{
  "topic": "string",
  "type": "string", 
  "timestamp": "number",
  "payload": "object"
}
```

* `topic`: The subscription topic (e.g., "crypto\_prices", "comments")
* `type`: The message type/event (e.g., "update", "reaction\_created")
* `timestamp`: Unix timestamp in milliseconds
* `payload`: Event-specific data object

## Subscription Management

### Subscribe to Topics

To subscribe to data streams, send a JSON message with this structure:

```json  theme={null}
{
  "action": "subscribe",
  "subscriptions": [
    {
      "topic": "topic_name",
      "type": "message_type",
      "filters": "optional_filter_string",
      "gamma_auth": {
        "address": "wallet_address"
      }
    }
  ]
}
```

### Unsubscribe from Topics

To unsubscribe from data streams, send a similar message with `"action": "unsubscribe"`.

## Error Handling

* Connection errors will trigger automatic reconnection attempts
* Invalid subscription messages may result in connection closure
* Authentication failures will prevent successful subscription to protected topics
