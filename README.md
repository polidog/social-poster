# social-poster

A small, fast CLI written in Rust that posts to **Misskey**, **Mastodon** and **Bluesky** at the same time.

- Posts to all accounts in parallel
- Plain CLI (args or stdin) and a minimal TUI composer
- One static binary, no async runtime

## Install

```sh
cargo install --path .
```

## Configure

Copy the example and fill in your credentials:

```sh
mkdir -p ~/.config/social-poster
cp config.example.toml ~/.config/social-poster/config.toml
chmod 600 ~/.config/social-poster/config.toml
```

The config lives at `$XDG_CONFIG_HOME/social-poster/config.toml` (default `~/.config/...`).
Set `SOCIAL_POSTER_CONFIG` to use another file.

Each `[[accounts]]` entry has a `name` (used by `--to`) and a `kind`:

| kind       | fields                                            | where to get the credential |
|------------|---------------------------------------------------|-----------------------------|
| `misskey`  | `host`, `token`                                   | Settings → API → Generate access token ("Compose or delete notes") |
| `mastodon` | `host`, `token`                                   | Preferences → Development → New application (`write:statuses`) |
| `bluesky`  | `identifier`, `password`, `host` (default `bsky.social`) | Settings → Privacy and security → App passwords |

You can list the same service more than once (e.g. two Misskey accounts), as long as each `name` is unique.

## Usage

```sh
social-poster "Hello, fediverse"            # post to every account
social-poster --to misskey,bsky "Hello"     # only these accounts
echo "from a pipe" | social-poster          # text from stdin
social-poster tui                           # interactive composer
```

Each account prints one line: `✓ name URL` on success or `✗ name error` on failure.
The exit code is non-zero if any post fails.

### TUI keys

| Key       | Action                   |
|-----------|--------------------------|
| Ctrl+S    | Post                     |
| Alt+1..9  | Toggle an account on/off |
| Enter     | New line                 |
| Esc       | Quit                     |

## Local testing

`dev/mock.py` fakes the three APIs on `localhost:8787` and prints every request it receives.
`host` accepts a full URL such as `http://localhost:8787`, so nothing is sent to real servers.

```sh
python3 dev/mock.py                                        # terminal 1
SOCIAL_POSTER_CONFIG=dev/config.toml cargo run -- "hello"  # terminal 2
```

## Limitations

- Text only: no images, content warnings or visibility settings yet
- Bluesky links and mentions are posted as plain text (no rich text facets)
- The TUI editor only edits at the end of the text

## Layout

```
src/
  main.rs          CLI, config, parallel posting
  tui.rs           TUI composer
  service/
    mod.rs         Service enum, error type, HTTP helper
    misskey.rs
    mastodon.rs
    bluesky.rs
dev/               mock server and config for local testing
```

## License

MIT
