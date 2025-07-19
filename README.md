# Converge Monitor

_(or, [Summer Of Making Monitor](https://github.com/SkyfallWasTaken/som-monitor) for Converge!)_

Converge Monitor is a monitor for Hack Club's [Converge YSWS](https://converge.hackclub.com). It continuously checks [the Emporium](https://emporium.hackclub.com) for item updates, keeping you in the loop on the stuff you want.

**Note:** If you want to see the code of the old version, which parsed the ESTree of the JS bundle of the [Converge website](https://converge.hackclub.com), see [the `estree-converge-site` branch](https://github.com/GGORG0/converge-monitor/tree/estree-converge-site).

## Development

0. [Get Rust](https://rustup.rs)
1. Make a `.env`

   ```sh
    BASE_URL=http://localhost:8000 # defaults to https://emporium.hackclub.com
    UPDATE_INTERVAL=10 # defaults to 300 (5 minutes)
    LOG_DIR=log # defaults to none, will log all changes in a computer-friendly format
    BLOCK_LOG_DIR=block_log # defaults to none, will log all sent Slack blocks
    SLACK_XOXB=xoxb-...
    SLACK_CHANNEL=... # channel id or #channel-name
    SLACK_USERGROUP_ID=... # optional
    COOKIE=session=... # your https://emporium.hackclub.com session cookie
   ```

2. `cargo run`
3. Or use Docker, pre-made image available at [`ghcr.io/ggorg0/converge-monitor:master`](https://github.com/ggorg0/converge-monitor/pkgs/container/converge-monitor) - see `compose.yaml`.
