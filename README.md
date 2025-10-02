# Webhook Service

A high-performance webhook receiver service built in Rust using Axum. Supports GitHub, GitLab, Bitbucket, Stripe, and generic webhooks with signature verification.

## Features

- **Multiple Platform Support**: GitHub, GitLab, Bitbucket, Stripe
- **Signature Verification**: HMAC-SHA256 signature validation
- **Event Storage**: In-memory event storage with configurable limits
- **RESTful API**: Retrieve stored webhook events
- **Health Checks**: Built-in health check endpoint
- **Docker Support**: Ready for containerized deployment

## Quick Start

### Local Development

```bash
cargo run
```

### Using Docker

```bash
docker-compose up --build
```

### Using Docker (standalone)

```bash
docker build -t webhook-service .
docker run -p 3000:3000 -e WEBHOOK_SECRET=your-secret webhook-service
```

## Configuration

Set environment variables or use command line flags:

- `BIND_ADDRESS`: Server bind address (default: `0.0.0.0:3000`)
- `WEBHOOK_SECRET`: Secret key for signature verification
- `MAX_EVENTS`: Maximum stored events (default: `1000`)
- `RUST_LOG`: Log level (default: `info`)

## API Endpoints

### Webhook Receivers

- `POST /webhook` - Generic webhook receiver
- `POST /webhook/{endpoint}` - Named endpoint webhook receiver

### Event Retrieval

- `GET /events` - List all stored webhook events
- `GET /events/{id}` - Get specific webhook event by ID
- `GET /health` - Health check endpoint

## Usage Examples

### GitHub Webhook

Configure GitHub webhook URL: `https://your-domain.com/webhook/github`

### GitLab Webhook

Configure GitLab webhook URL: `https://your-domain.com/webhook/gitlab`

### Generic Webhook

Use: `https://your-domain.com/webhook`

## Deployment

### Railway

```bash
railway login
railway init
railway up
```

### Fly.io

```bash
flyctl auth login
flyctl launch
flyctl deploy
```

### Heroku

```bash
heroku create your-webhook-service
heroku container:push web
heroku container:release web
```

## Security

- Always use HTTPS in production
- Set a strong `WEBHOOK_SECRET` for signature verification
- Consider rate limiting and authentication for production use