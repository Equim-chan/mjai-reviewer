# akochan-reviewer

[![GitHub Workflow Status](https://github.com/Equim-chan/akochan-reviewer/workflows/build/badge.svg)](https://github.com/Equim-chan/akochan-reviewer/actions)
[![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/Equim-chan/akochan-reviewer?include_prereleases)](https://github.com/Equim-chan/akochan-reviewer/releases)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Equim-chan/akochan-reviewer)
[![License](https://img.shields.io/github/license/Equim-chan/akochan-reviewer)](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

Review your Tenhou or Mahjong Soul (Jantama) log with mahjong AI akochan.

[***Try it online!***](https://akochan.ekyu.moe) | [Demo result page](https://gh.ekyu.moe/akochan-reviewer-demo.html)

**This tool is still in early stages. There are still lots of features to be implemented, and breaking changes may be made at anytime. Suggestions and contributions are welcome. 日本語もおｋ.**

### [How to Review Mahjong Soul Logs (updated 2021-09-26)](https://github.com/Equim-chan/akochan-reviewer/blob/master/mjsoul.adoc)

## Example
```console
$ # Review https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2
$ # Note that you may need to quote it in the shell to escape the string
$ akochan-reviewer "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Review https://game.mahjongsoul.com/?paipu=190425-146daa2a-68c2-4b7e-a8d7-2b5c71b54b00_a458023588

$ # Alternatively, you can specify the log ID and actor manually
$ akochan-reviewer -t 2019050417gm-0029-0000-4f2a8622 -a 2

$ # Review with arbitrary pt distribution
$ akochan-reviewer --pt 75,30,0,-165 "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Review with arbitrary pt distribution under the acceptance of <=0.05 pt deviation
$ akochan-reviewer --pt 75,30,0,-165 -n 0.05 "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Review with placement EV instead of pt EV
$ akochan-reviewer -e "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Review with placement EV instead of pt EV under the acceptance of <=0.002 placement deviation
$ akochan-reviewer -e -n 0.002 "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

$ # Review 東2局1本場 and 東3局 only
$ akochan-reviewer -k E2.1,E3 "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"
```

## Usage
```plain
USAGE:
    akochan-reviewer.exe [FLAGS] [OPTIONS] [URL]

FLAGS:
        --anonymous           Do not include player names.
    -h, --help                Prints help information
        --json                Output review result in JSON instead of HTML.
        --no-open             Do not open the output file in browser after finishing.
        --no-review           Do not review at all. Only download and save files.
    -e, --use-placement-ev    Use final placement EV instead of pt EV. This will override --pt and "jun_pt" in
                              --tactics-config.
    -V, --version             Prints version information
    -v, --verbose             Use verbose output.
        --without-viewer      Do not include log viewer in the generated HTML report.

OPTIONS:
    -a, --actor <INDEX>                      Specify the actor to review. It is the number after "&tw=" in tenhou's log
                                             url.
    -d, --akochan-dir <DIR>                  Specify the directory of akochan. This will serve as the working directory
                                             of akochan process. Default value "akochan".
    -n, --deviation-threshold <THRESHOLD>    THRESHOLD is an absolute value that the reviewer will ignore all
                                             problematic moves whose EVs are within the range of [best EV - THRESHOLD,
                                             best EV]. This option is effective under both pt and placement EV mode. It
                                             is recommended to use it with --use-placement-ev where the reward
                                             distribution is fixed and even. Reference value: 0.05 when using pt and
                                             0.001 when using placement. Default value: "0.001".
    -i, --in-file <FILE>                     Specify a tenhou.net/6 format log file to review. If FILE is "-" or empty,
                                             read from stdin.
    -k, --kyokus <LIST>                      Specify kyokus to review. If LIST is empty, review all kyokus. Format:
                                             "E1,E4,S3.1".
        --lang <LANG>                        Set the language for the rendered report page. Default value "ja".
                                             Supported languages: ja, en.
        --mjai-out <FILE>                    Save the transformed mjai format log to FILE. If FILE is "-", write to
                                             stdout.
    -m, --mjsoul-id <ID>                     Specify a Mahjong Soul log ID to review. Example: "200417-e1f9e08d-487f-
                                             4333-989f-34be08b943c7".
        --out-dir <DIR>                      Specify a directory to save the output for mjai logs. If DIR is empty,
                                             defaults to ".".
    -o, --out-file <FILE>                    Specify the output file for generated HTML report. If FILE is "-", write to
                                             stdout; if FILE is empty, write to "{tenhou_id}&tw={actor}.html" if
                                             --tenhou-id is specified, otherwise "report.html".
        --pt <LIST>                          Shortcut to override "jun_pt" in --tactics-config. Format: "90,45,0,-135".
    -c, --tactics-config <FILE>              Specify the tactics config file for akochan. Default value "tactics.json".
    -t, --tenhou-id <ID>                     Specify a Tenhou log ID to review, overriding --in-file. Example:
                                             "2019050417gm-0029-0000-4f2a8622".
        --tenhou-ids-file <FILE>             Specify a file of Tenhou log ID list to convert to mjai format, implying
                                             --no-review.
        --tenhou-out <FILE>                  Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is
                                             specified. If FILE is "-", write to stdout.

ARGS:
    <URL>    Tenhou or Mahjong Soul log URL.
```

## Build
### Build akochan
First of all, build [akochan](https://github.com/critter-mj/akochan).

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

### Build akochan-reviewer
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
$ git clone https://github.com/critter-mj/akochan.git
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

Under Powershell
```console
> $env:OMP_NUM_THREADS = 8
```

Under MSYS2 bash
```console
$ export OMP_NUM_THREADS=8
```

## Acknowledgment
* [critter](https://twitter.com/critter_Eng): The creater of akochan, who also proposed many advise and gave help to the development of akochan-reviewer.
* [新篠ゆう](https://github.com/yuarasino): Who helped a lot with the report page template.
* [Amber](https://euophrys.itch.io/): English translater of the report page, also has written a [blog post](https://pathofhouou.blogspot.com/2020/08/training-tool-ako-chan-reviewer.html) about akochan-reviewer.

## License
[Apache-2.0](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

akochan itself was licensed separately, see https://github.com/critter-mj/akochan/blob/master/LICENSE.

## Contributors ✨

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="https://github.com/critter-mj"><img src="https://pbs.twimg.com/profile_images/1005709404623155201/kMTv4X6X_400x400.jpg?s=100" width="100px;" alt=""/><br /><sub><b>critter-mj</b></sub></a><br /><a href="#ideas-critter-mj" title="Ideas, Planning, & Feedback">🤔</a> <a href="#tool-critter-mj" title="Tools">🔧</a> <a href="#data-critter-mj" title="Data">🔣</a></td>
    <td align="center"><a href="https://github.com/yuarasino"><img src="https://avatars2.githubusercontent.com/u/37145593?v=4?s=100" width="100px;" alt=""/><br /><sub><b>新篠ゆう</b></sub></a><br /><a href="#ideas-yuarasino" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/Equim-chan/akochan-reviewer/commits?author=yuarasino" title="Code">💻</a> <a href="#design-yuarasino" title="Design">🎨</a></td>
    <td align="center"><a href="https://euophrys.itch.io/"><img src="https://avatars0.githubusercontent.com/u/47927161?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Amber</b></sub></a><br /><a href="#translation-Euophrys" title="Translation">🌍</a> <a href="#blog-Euophrys" title="Blogposts">📝</a></td>
  </tr>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->
