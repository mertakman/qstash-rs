
# Upstash QStash Rust SDK

**QStash** is a robust, HTTP-based messaging and scheduling solution optimized for serverless and edge runtimes. With a stateless, HTTP-driven design, it supports a broad range of environments and platforms, including:

-   Serverless functions (e.g., AWS Lambda) – [Example](https://github.com/mertakman/qstash-rs/tree/main/examples/aws-lambda/main.rs)
-   Cloudflare Workers – [Example](https://github.com/mertakman/qstash-rs/tree/main/examples/cloudflare-workers/main.rs)
-   Fastly Compute@Edge
-   Next.js, including [Edge runtime](https://nextjs.org/docs/api-reference/edge-runtime)
-   Deno
-   Client-side web and mobile applications
-   WebAssembly
-   Any other environment where HTTP-based communication is preferred over TCP

## How QStash Works

QStash serves as the intermediary message broker for serverless applications. By sending a simple HTTP request to QStash, you can include a destination, payload, and optional configurations. QStash then stores the message securely and reliably delivers it to the designated API endpoint. In cases where the destination is temporarily unavailable, QStash ensures at-least-once delivery by automatically retrying until the message is successfully received.

## Quick Start

### 1. Obtain Your Authorization Token

To get started, head to the [Upstash Console](https://console.upstash.com/qstash) and copy your **QSTASH_TOKEN**.

### 2. Explore Examples

For API documentation and a quickstart guide, refer to the official [QStash API Documentation](https://upstash.com/docs/qstash/api/). Below, you'll find links to additional examples that demonstrate usage for each endpoint:

-   **Dead Letter Queue** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/dead_letter_queue/main.rs)
-   **Events** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/events/main.rs)
-   **LLM** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/llm/main.rs)
-   **Messages** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/messages/main.rs)
-   **Queues** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/queues/main.rs)
-   **Schedulers** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/schedulers/main.rs)
-   **Signing Keys** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/signing_keys/main.rs)
-   **URL Groups** – [Example](https://github.com/mertakman/qstash-rs/blob/main/examples/url_groups/main.rs)

## Supported Environments

QStash is ideal for use with serverless architectures and edge deployments, supporting scenarios where HTTP-based communication provides flexibility and compatibility with modern applications.
