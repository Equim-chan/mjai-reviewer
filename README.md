# mjai-reviewer

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Equim-chan/mjai-reviewer/build.yml)](https://github.com/Equim-chan/mjai-reviewer/actions)
[![dependency status](https://deps.rs/repo/github/Equim-chan/mjai-reviewer/status.svg)](https://deps.rs/repo/github/Equim-chan/mjai-reviewer)
![GitHub top language](https://img.shields.io/github/languages/top/Equim-chan/mjai-reviewer)
![Lines of code](https://www.aschey.tech/tokei/github/Equim-chan/mjai-reviewer)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Equim-chan/mjai-reviewer)
[![License](https://img.shields.io/github/license/Equim-chan/mjai-reviewer)](https://github.com/Equim-chan/mjai-reviewer/blob/master/LICENSE)

[![Donate](https://img.shields.io/badge/Donate-%E2%9D%A4%EF%B8%8F-blue?style=social)](https://mortal.ekyu.moe/donate.html)

Review your mahjong gameplay with the help of mjai-compatible mahjong AI engines, including [Mortal](https://github.com/Equim-chan/Mortal) and [akochan](https://github.com/critter-mj/akochan).

**[Try it online](https://mjai.ekyu.moe)** | [Demo result page](https://gh.ekyu.moe/mjai-reviewer-demo.html)

It is recommended to just use the [web app](https://mjai.ekyu.moe), which works for Mahjong Soul games out-of-the-box, no download, no install, no extension, and it is free to use.

mjai-reviewer 1.x.x is incompatible with 0.x.x versions, which were previously known as akochan-reviewer. If you prefer the old version, check out [v0 branch](https://github.com/Equim-chan/mjai-reviewer/tree/v0).

[Guide on reviewing mahjong soul logs locally](https://github.com/Equim-chan/mjai-reviewer/blob/master/mjsoul.adoc)

## Usage
```console
$ # Review https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2
$ # Note that you may need to quote it in the shell to escape the string
$ mjai-reviewer -e mortal -u "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Use akochan as engine
$ mjai-reviewer -e akochan -u "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Alternatively, you can specify the log ID and player ID manually
$ mjai-reviewer -e mortal -t 2019050417gm-0029-0000-4f2a8622 -a 2

$ # Review a local tenhou.net/6 format log file, note that you must specify a player ID
$ mjai-reviewer -e mortal -i log.json -a 3

$ # Review 東2局1本場 and 東3局 only
$ mjai-reviewer -e mortal -k E2.1,E3 -u "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"
```

Use the `--help` argument for more details.

## FAQ
See [FAQ](https://github.com/Equim-chan/mjai-reviewer/blob/master/faq.md).

## Troubleshooting
### (akochan) `Assertion failed` errors on Windows
Set environment variable `OMP_NUM_THREADS=8`.

Under cmd
```console
> set OMP_NUM_THREADS=8
```

Under Powershell
```console
> $env:OMP_NUM_THREADS = 8
```

Under MSYS2 bash
```console
$ export OMP_NUM_THREADS=8
```

### (akochan) `libai.so` not found on Linux
Try adding the directory of `libai.so` to env `LD_LIBRARY_PATH`.


## Build
### mjai-reviewer
Follow the instructions [here](https://www.rust-lang.org/learn/get-started) to install Rust toolchains first, if you haven't yet.

```console
$ cd ..
$ git clone https://github.com/Equim-chan/mjai-reviewer.git
$ cargo build --release
```

`mjai-reviewer` binary will be in `target/release` directory.

### Engines
#### Mortal
See [Mortal's documentation](https://mortal.ekyu.moe/user/build.html).

You also need a trained model file to actually use Mortal.

#### Akochan
```console
$ git clone https://github.com/critter-mj/akochan.git
$ cd akochan
```

You have to edit `Makefile` and `ai_src/Makfefile` accordingly. Set up correct path for boost and some other options like `-march=native` of your choice.

<details><summary>On Windows MSYS2 with MinGW-w64 toolchain</summary>
<p>

```console
$ pacman -Syu mingw-w64-x86_64-{toolchain,boost}
```

Edit `Makefile`:

```Makefile
LIBS = -lboost_system-mt -lws2_32 -L./ -lai -s
```

Edit `ai_src/Makefile`:

```Makefile
LIBS = -lboost_system-mt -lws2_32
```

```console
$ cd ai_src
$ make
$ cd ..
$ make
```

</p>
</details>

<details><summary>On MacOS</summary>
<p>

```console
$ brew install llvm libomp boost
$ cd ai_src
$ make -f Makefile_MacOS
$ cd ..
$ make -f Makefile_MacOS
```

</p>
</details>

<details><summary>On Arch Linux</summary>
<p>

```console
$ sudo pacman -Syu base-devel boost
$ make -f Makefile_Linux
$ cd ..
$ make -f Makefile_Linux
```

</p>
</details>

## Docker
Currently the docker image is not maintained and it only embeds akochan engine.

### Build
```console
$ git clone https://github.com/Equim-chan/mjai-reviewer.git
$ cd mjai-reviewer
$ git clone https://github.com/critter-mj/akochan.git
$ docker build -t mjai-reviewer:latest .
```

### Usage
```console
$ docker run --rm mjai-reviewer:latest -e akochan --no-open -t 2019050417gm-0029-0000-4f2a8622 -a 3 -o - > report.html
$ open report.html  # or just open in browser
```

## License
[Apache-2.0](https://github.com/Equim-chan/mjai-reviewer/blob/master/LICENSE)

## Contributors
[![Contributors](https://contrib.rocks/image?repo=Equim-chan/mjai-reviewer)](https://github.com/Equim-chan/mjai-reviewer/graphs/contributors)
