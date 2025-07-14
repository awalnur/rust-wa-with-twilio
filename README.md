# Chatbot-WA

A Rust-based WhatsApp chatbot application using Actix Web framework with Google Sheets integration.

## Overview

This project implements a web service that can interact with WhatsApp API and Google Sheets. It's built with Rust and uses the Actix Web framework for handling HTTP requests.

## Features

- RESTful API endpoints using Actix Web
- Google Sheets integration for data storage/retrieval via google-sheets4
- Asynchronous operations with Tokio
- HTTP client functionality with reqwest
- JSON serialization/deserialization with serde
- Environment configuration using dotenv

## Prerequisites

- Rust (latest stable version)
- Google Cloud service account credentials

## Setup

1. Clone the repository
2. Configure your environment variables (using `.env` file)
3. Make sure your Google service account credentials are properly set up

## Installation

```bash
# Install dependencies
cargo build

# Run the application
cargo run
```

## Configuration

The application uses environment variables for configuration, which can be loaded from a `.env` file using the dotenv crate.

### Environment Variables Example

Create a `.env` file in the root directory with the following variables:

```
# Server Configuration
PORT=8080
HOST=127.0.0.1

# Google Sheets API Configuration
GOOGLE_APPLICATION_CREDENTIALS=./vivid-motif-388517-8f97fd6b44c5.json
SPREADSHEET_ID=your-spreadsheet-id
SHEET_NAME=your-sheet-name

# WhatsApp API Configuration
WHATSAPP_API_TOKEN=your-whatsapp-api-token
WHATSAPP_PHONE_NUMBER_ID=your-phone-number-id

# Optional: Logging Configuration
RUST_LOG=info

# Add any other environment variables required for your specific implementation
```

Make sure to replace the placeholder values with your actual configuration.

## License

[MIT](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
