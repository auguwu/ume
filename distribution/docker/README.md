# 🐻‍❄️💐 ume
> *Easy, self-hostable, and flexible image host made in Rust*

**ume** is a simple and flexible image host to be used with ShareX or Flameshot to provide a blazing-fast way to send images to your friends on various different platforms like Discord, Telegram, etc.

**ume**'s plan is to be minimal as possible when self-hosting, so no external configuration is *required* but is there if you wish to customize every aspect of **ume**.

## Usage
This will spawn a Ume server that will listen under `:3621`:

```shell
$ docker run --name ume -e UME_UPLOADER_KEY="a random string" -d -p 3621:3621 auguwu/ume
```

The `ume` CLI is available as the Docker image, you can overwrite the default `CMD` instruction:

```shell
$ docker run --rm auguwu/ume ume -h
```

and will print the following:

```shell
🐻‍❄️💐 Easy, self-hostable, and flexible image host made in Rust

Usage: ume <COMMAND> [...ARGS]

Commands:
  completions  Generates shell completions for any shell. This doesn't support nushell or fig
  screenshot   Takes a screenshot with [Flameshot](https://flameshot.org)
  server       Starts a Ume server
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --log-level <LEVEL>  Configures the log level for all CLI-based commands. This will not configure the Ume server's log level when you run `ume server` [default: INFO]
  -q, --quiet              suppress all log output even when `--log-level` is specified
      --no-progress        disables the use of progress bars in `ume screenshot` to indicate that a file is uploading to the server [env: UME_NO_PROGRESSBAR=]
  -h, --help               Print help
```
