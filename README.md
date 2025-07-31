# ShellHook: Stream Command Output to Webhooks

[![CI](https://github.com/r0king/shell_hook/actions/workflows/ci.yml/badge.svg)](https://github.com/r0king/shell_hook/actions/workflows/ci.yml)

**ShellHook** is a powerful and flexible CLI tool that bridges the gap between your command-line scripts and modern notification platforms. It captures the `stdout` and `stderr` of any command, buffers the output, and streams it to a webhook in real-time.

It's perfect for monitoring long-running processes, getting build notifications, or receiving alerts from cron jobs.

## Key Features

- **Real-time Output Streaming**: Get instant feedback from your scripts by streaming `stdout` and `stderr` directly to your preferred webhook.
  
  ![Demo](https://github.com/r0king/shell_hook/assets/18419334/531d0411-92ba-4475-b072-a08b5f259695)

- **Interactive Shell Mode**: Launch a persistent session for running multiple commands without exiting.
- **Smart Buffering**: Avoid rate-limiting issues with intelligent line buffering. Output is sent in batches based on size or time, ensuring you never miss a line.
- **Customizable Messages**: Tailor notifications for command success or failure. Provide context and clarity with custom titles and messages.
- **Webhook Agnostic**: Supports popular webhook formats like Google Chat and Slack out of the box.
- **Quiet Mode**: Suppress noisy command output and receive only essential start and finish notifications.
- **Dry Run Mode**: Test your configuration without executing commands or sending webhooks.
- **Environment Variable Support**: Easily configure the webhook URL via the `WEBHOOK_URL` environment variable for seamless integration with CI/CD pipelines.

## Why ShellHook?

In a world of automated workflows and CI/CD pipelines, getting timely notifications is crucial. ShellHook was built to solve a common problem: how do you easily monitor the output of a command-line script from a remote system?

With ShellHook, you can:
- **Monitor cron jobs**: Get alerts if your nightly backups fail.
- **Track deployments**: See the progress of your deployment script in real-time.
- **Stream build logs**: Keep an eye on your CI/CD pipeline from your favorite chat client.

## Installation

### From source

1.  Clone the repository:
    ```sh
    git clone https://github.com/r0king/shell_hook.git
    cd shell_hook
    ```
2.  Build and install:
    ```sh
    cargo install --path .
    ```

## Quick Start

1.  **Set the webhook URL**:
    ```sh
    export WEBHOOK_URL="https://your-webhook-url"
    ```
2.  **Run a command**:
    ```sh
    shell_hook run --title "My First Job" --on-success "It worked! ✅" -- ls -la
    ```

This will run `ls -la`, stream its output to your webhook, and send a "It worked! ✅" message upon completion.

## Usage

### Run a single command

```sh
shell_hook run [OPTIONS] -- <COMMAND>...
```

### Start an interactive shell

```sh
shell_hook shell
```

## Options

### Global Options

| Option | Environment Variable | Description |
|---|---|---|
| `--webhook-url <URL>` | `WEBHOOK_URL` | The webhook URL to send messages to. |
| `-t`, `--title <TITLE>` | | A title to prepend to all messages (e.g., "[My Project]"). |
| `--dry-run` | | Don't execute the command or send webhooks. |
| `--format <FORMAT>` | | Webhook payload format. (Options: `google-chat`, `slack`) |

### `run` Subcommand Options

| Option | Description |
|---|---|
| `--on-success <MSG>` | Custom message to send on command success. |
| `--on-failure <MSG>` | Custom message to send on command failure. |
| `-q`, `--quiet` | Suppress streaming of stdout/stderr to the webhook. |
| `<COMMAND>...` | The command to execute and stream. |

## Webhook Formats

-   `google-chat`: Formats the payload for Google Chat webhooks. (Default)
-   `slack`: Formats the payload for Slack webhooks.

## Contributing

Contributions are welcome! If you have a feature request, bug report, or pull request, please feel free to open an issue or submit a PR.

## License

This project is licensed under the MIT License.