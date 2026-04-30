<p align="center">
  <a href="https://openwarp.zerx.dev/en/">Website</a>
  ·
  <a href="https://openwarp.zerx.dev/en/">Docs</a>
</p>

## About

[OpenWarp](https://openwarp.zerx.dev/en/) is an agentic development environment, born out of the terminal. Use OpenWarp's built-in coding agent, or bring your own CLI agent (Claude Code, Codex, Gemini CLI, and others).

This repository is **not affiliated with, endorsed by, or sponsored by Warp**. It is an independent project.

## Installation

You can [download OpenWarp](https://openwarp.zerx.dev/en/) and read the docs for platform-specific instructions.

## Licensing

OpenWarp's UI framework (the `warpui_core` and `warpui` crates) are licensed under the [MIT license](LICENSE-MIT).

The rest of the code in this repository is licensed under the [AGPL v3](LICENSE-AGPL).

## Open Source & Contributing

OpenWarp's client codebase is open source and lives in this repository. We welcome community contributions and have designed a lightweight workflow to help new contributors get started. For the full contribution flow, read our [CONTRIBUTING.md](CONTRIBUTING.md) guide.

### Issue to PR

Before filing, search existing issues for your bug or feature request. If nothing exists, file an issue using our templates. Security vulnerabilities should be reported privately as described in [CONTRIBUTING.md](CONTRIBUTING.md#reporting-security-issues).

Once filed, an OpenWarp maintainer reviews the issue and may apply a readiness label: `ready-to-spec` signals the design is open for contributors to spec out, and `ready-to-implement` signals the design is settled and code PRs are welcome. Anyone can pick up a labeled issue — mention **@oss-maintainers** on an issue if you'd like it considered for a readiness label.

### Building the Repo Locally

To build and run OpenWarp from source:

```bash
./script/bootstrap   # platform-specific setup
./script/run         # build and run OpenWarp
./script/presubmit   # fmt, clippy, and tests
```

See [WARP.md](WARP.md) for the full engineering guide, including coding style, testing, and platform-specific notes.

## Support and Questions

1. See the [OpenWarp site](https://openwarp.zerx.dev/en/) for a guide to OpenWarp's features.
2. Mention **@oss-maintainers** on any issue to escalate to the team — for example, if you encounter problems with the automated agents.

## Code of Conduct

We ask everyone to be respectful and empathetic. OpenWarp follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Open Source Dependencies

We'd like to call out a few of the open source dependencies that have helped OpenWarp to get off the ground:

* [Tokio](https://github.com/tokio-rs/tokio)
* [NuShell](https://github.com/nushell/nushell)
* [Fig Completion Specs](https://github.com/withfig/autocomplete)
* [Warp Server Framework](https://github.com/seanmonstar/warp)
* [Alacritty](https://github.com/alacritty/alacritty)
* [Hyper HTTP library](https://github.com/hyperium/hyper)
* [FontKit](https://github.com/servo/font-kit)
* [Core-foundation](https://github.com/servo/core-foundation-rs)
* [Smol](https://github.com/smol-rs/smol)
