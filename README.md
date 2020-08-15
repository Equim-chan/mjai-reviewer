# akochan-reviewer

[![GitHub Workflow Status](https://github.com/Equim-chan/akochan-reviewer/workflows/build/badge.svg)](https://github.com/Equim-chan/akochan-reviewer/actions)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Equim-chan/akochan-reviewer)
[![License](https://img.shields.io/github/license/Equim-chan/akochan-reviewer)](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

Review your Tenhou log with mahjong AI akochan.

[demo](https://gh.ekyu.moe/akochan-reviewer-demo.html)

**This tool is still in early stages. There are still lots of features to be implemented, and breaking changes may be made at anytime. Suggestions and contributions are welcome. 日本語もおｋ.**

## Example
```console
$ # Review https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2
$ akochan-reviewer -t 2019050417gm-0029-0000-4f2a8622 -a 2

$ # Review with arbitrary pt distribution
$ akochan-reviewer -t 2019050417gm-0029-0000-4f2a8622 -a 2 --pt 75,30,0,-165

$ # Review every move, including moves that already match akochan's choice
$ akochan-reviewer -t 2019050417gm-0029-0000-4f2a8622 -a 2 -f

# Review 東2局1本場 and 東3局 only
$ akochan-reviewer -t 2019050417gm-0029-0000-4f2a8622 -a 2 -k E2.1,E3
```

## Usage
```plain
USAGE:
    akochan-reviewer [FLAGS] [OPTIONS]

FLAGS:
    -f, --full               Analyze every move, not only the different ones.
    -h, --help               Prints help information
        --json               Output review result in JSON instead of HTML
        --no-open            Do not open the output file in browser after finishing
        --no-review          Do not review at all. Only download and save files
        --use-ranking-exp    Use final ranking exp instead of pt exp. This will override --pt and "jun_pt" in --tactics-
                             config.
    -V, --version            Prints version information
    -v, --verbose            Use verbose output
        --without-viewer     Do not include log viewer in the generated HTML report

OPTIONS:
    -a, --actor <INDEX>             Specify the actor to review. It is the number after "&tw=" in tenhou's log url
    -d, --akochan-dir <DIR>         Specify the directory of akochan. This will serves as the working directory of
                                    akochan process. Default value "akochan"
    -i, --in-file <FILE>            Specify a tenhou.net/6 format log file to review. If FILE is "-" or empty, read from
                                    stdin
    -k, --kyokus <ARRAY>            Specify kyokus to review. If ARRAY is empty, review all kyokus. Format: "E1,E4,S3.1"
        --mjai-out <FILE>           Save the transformed mjai format log to FILE. If FILE is "-", write to stdout
        --out-dir <DIR>             Specify a directory to save the output for mjai logs. If DIR is empty, defaults to
                                    "."
    -o, --out-file <FILE>           Specify the output file for generated HTML report. If FILE is "-", write to stdout;
                                    if FILE is empty, write to "{tenhou_id}&tw={actor}.html" if --tenhou-id is
                                    specified, otherwise "report.html"
        --pt <ARRAY>                Shortcut to override "jun_pt" in --tactics-config. Format: "90,45,0,-135"
    -c, --tactics-config <FILE>     Specify the tactics config file for akochan. Default value "tactics.json"
    -t, --tenhou-id <ID>            Specify a Tenhou log ID to review, overriding --in-file. Example: "2019050417gm-
                                    0029-0000-4f2a8622"
        --tenhou-ids-file <FILE>    Specify a file of Tenhou log ID list to convert to mjai format, implying --no-
                                    review.
        --tenhou-out <FILE>         Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is specified.
                                    If FILE is "-", write to stdout
```

## Build
### Build akochan
First of all, build [akochan of my fork](https://github.com/Equim-chan/akochan).

```console
$ git clone https://github.com/Equim-chan/akochan.git
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
LIBS = -L/mingw64/lib/boost -lws2_32 -L./ -lai
```

Edit `ai_src/Makefile`:

```Makefile
LIBS = -L/mingw64/lib/boost -lws2_32
```

```console
$ cd ai_src
$ make ai.dll
$ cd ..
$ make system.exe
```

</p>
</details>

<details><summary>On MacOS</summary>
<p>

```console
$ brew install llvm libomp boost
```

Edit `Makefile_Linux`:

```Makefile
COMPILER = /usr/local/opt/llvm/bin/clang++
CFLAGS = -g -MMD -MP -std=c++11 -O3 -fopenmp -I/usr/local/include -I./
LIBS = -L/usr/local/lib -lboost_system -L./ -lai
```

Edit `ai_src/Makefile_Linux`:

```Makefile
COMPILER = /usr/local/opt/llvm/bin/clang++
CFLAGS = -g -MMD -MP -std=c++11 -O3 -fopenmp -I/usr/local/include
LIBS = -L/usr/local/lib -lboost_system
```

```console
$ cd ai_src
$ make -f Makefile_Linux libai.so
$ cd ..
$ make -f Makefile_Linux system.exe
```

</p>
</details>

<details><summary>On Arch Linux</summary>
<p>

```console
$ sudo pacman -Syu base-devel boost
$ make -f Makefile_Linux libai.so
$ cd ..
$ make -f Makefile_Linux system.exe
```

</p>
</details>

### Build akochan-review
Follow the instructions [here](https://www.rust-lang.org/learn/get-started) to install Rust toolchains first, if you haven't yet.

```console
$ cd ..
$ git clone https://github.com/Equim-chan/akochan-reviewer.git
$ export RUSTFLAGS="-C target-cpu=native" # optional
$ cargo build --release
```

`akochan-reviewer` binary will be in `target/release` directory.

## Docker
### Build
```console
$ git clone https://github.com/Equim-chan/akochan-reviewer.git
$ cd akochan-reviewer
$ git clone https://github.com/Equim-chan/akochan.git
$ docker build -t akochan-reviewer:latest .
```

### Usage
```console
$ docker run --rm akochan-reviewer:latest --no-open -t 2019050417gm-0029-0000-4f2a8622 -a 3 -o - > report.html
$ open report.html  # or just open in browser
```

## Troubleshooting
### `Assertion failed` errors on Windows
Set environment variable `OMP_NUM_THREADS=8`.

Under cmd
```console
> set OMP_NUM_THREADS=8
```

under MSYS2
```console
$ export OMP_NUM_THREADS=8
```

## Acknowledgment
* [critter](https://twitter.com/critter_Eng): The creater of akochan, who also proposed many advise and gave help to the development of akochan-reviewer.

## License
[Apache-2.0](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

akochan itself was licensed separately, see https://github.com/critter-mj/akochan/blob/master/LICENSE.

## Contributors ✨

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="https://github.com/critter-mj"><img src="https://pbs.twimg.com/profile_images/1005709404623155201/kMTv4X6X_400x400.jpg" width="100px;" alt=""/><br /><sub><b>critter-mj</b></sub></a><br /><a href="#ideas-critter-mj" title="Ideas, Planning, & Feedback">🤔</a> <a href="#tool-critter-mj" title="Tools">🔧</a> <a href="#data-critter-mj" title="Data">🔣</a></td>
    <td align="center"><a href="https://github.com/yuarasino"><img src="https://avatars2.githubusercontent.com/u/37145593?v=4" width="100px;" alt=""/><br /><sub><b>新篠ゆう</b></sub></a><br /><a href="#ideas-yuarasino" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/Equim-chan/akochan-reviewer/commits?author=yuarasino" title="Code">💻</a> <a href="#design-yuarasino" title="Design">🎨</a></td>
  </tr>
</table>

<!-- markdownlint-enable -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->
