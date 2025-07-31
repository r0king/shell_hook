# hook-stream

A powerful CLI tool to stream command output to webhooks with buffering, custom messages, and multi-platform support.

## Features

- **Real-time Output Streaming**: Stream `stdout` and `stderr` of any command to a webhook.
- **Buffering**: Lines are buffered and sent in batches to avoid rate-limiting.
- **Customizable Messages**: Set custom messages for command success or failure.
- **Webhook Formats**: Supports different webhook payload formats like Google Chat and Slack.
- **Quiet Mode**: Option to suppress command output and only send start/finish messages.
- **Dry Run**: Test your command without executing or sending webhooks.
- **Environment Variable Support**: Configure the webhook URL via the `WEBHOOK_URL` environment variable.

## Installation

### From source

1.  Clone the repository:
    ```sh
    git clone https://github.com/royal-babu/hook-stream.git
    cd hook-stream
    ```
2.  Build and install:
    ```sh
    cargo install --path .
    ```

## Usage

```sh
hook-stream [OPTIONS] -- <COMMAND>...
```

## Options

| Option                | Environment Variable | Description                                                                                             |
| --------------------- | -------------------- | ------------------------------------------------------------------------------------------------------- |
| `--webhook-url <URL>` | `WEBHOOK_URL`        | The webhook URL to send messages to.                                                                    |
| `--on-success <MSG>`  |                      | Custom message to send on command success.                                                              |
| `--on-failure <MSG>`  |                      | Custom message to send on command failure.                                                              |
| `-q`, `--quiet`       |                      | Suppress streaming of stdout/stderr to the webhook (start/finish messages are still sent).              |
| `-t`, `--title <TITLE>` |                      | A title to prepend to all messages, e.g., "[My Project]".                                               |
| `--dry-run`           |                      | Don't execute the command or send webhooks; just print what would be done.                              |
| `--format <FORMAT>`   |                      | The format of the webhook payload. (Options: `google-chat`, `slack`) (Default: `google-chat`) |
| `<COMMAND>...`        |                      | The command to execute and stream its output.                                                           |

## Webhook Formats

You can specify the webhook format using the `--format` option.

-   `google-chat`: Formats the payload for Google Chat webhooks. (Default)
-   `slack`: Formats the payload for Slack webhooks.

## Examples

### Basic Usage

Stream the output of a simple `echo` command.

```sh
export WEBHOOK_URL="https://your-webhook-url"
hook-stream -- ls -la
```

### With a Title

Add a title to all messages sent to the webhook.

```sh
hook-stream --title "My Awesome Project" -- ls -la
```

### Custom Success and Failure Messages

Send custom messages depending on the command's exit code.

```sh
hook-stream --on-success "Deployment complete! 🎉" --on-failure "Deployment failed. 😢" -- ./deploy.sh
```

### Using a Different Webhook Format

Send output to a Slack webhook.

```sh
hook-stream --format slack --webhook-url "https://hooks.slack.com/services/..." -- echo "Hello from hook-stream!"
```

### Quiet Mode

Run a command but only get notified when it starts and finishes, not with the full output.

```sh
hook-stream --quiet -- ./long-running-script.sh
```

### Dry Run

See what the tool would do without actually running the command or sending a webhook.

```sh
hook-stream --dry-run --on-success "This will not be sent" -- echo "This will not run"
```

## License

This project is licensed under the MIT License.