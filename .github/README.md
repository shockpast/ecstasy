<div align="center">
  <h3><a href="https://github.com/shockpast">~shockpast/</a>ecstasy</h3>
  <p>downloader for <a href="https://osucollector.com">osu!collector</a></p>
</div>

## Installation

1. download the latest release *(or compile yourself)*
2. setup the `config.toml` (`config.toml.example`, you'll need to rename it)
3. launch `ecstasy.exe`

## Building

```sh
git clone https://github.com/shockpast/ecstasy
cd ecstasy
cargo build --release --target=<desired_target>
```

target triplets:
```sh
x86_64-unknown-linux-gnu # linux 64-bit
x86_64-pc-windows-msvc # windows 64-bit
i686-unknown-linux-gnu # linux 32-bit
i686-pc-windows-msvc # windows 32-bit
```

## Tips

- `ecstasy.exe -s` will run SpeedTest against all osu! mirrors, and also a general test for download.

## Todo

- [ ] osu!lazer support