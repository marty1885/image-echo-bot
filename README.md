# image-echo-bot

A simple bot that echoes back images sent to it.

## Running

```bash
export DISCORD_TOKEN=your_token
cargo run --release
```

## Bot Commands

- `!echo` - Bot sends back some text to test if it's working.
- `!begin` - Bot starts recoding images sent to it.
- `!end` - Bot stops recording and sends back all images it has recorded.
- `!extract <url_to_message>` - Bot sends back all images in the message.