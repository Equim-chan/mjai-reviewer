# akochan-reviewer

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Equim-chan/akochan-reviewer/Rust)](https://github.com/Equim-chan/akochan-reviewer/actions)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Equim-chan/akochan-reviewer)
[![License](https://img.shields.io/github/license/Equim-chan/akochan-reviewer)](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

Review your Tenhou log with mahjong AI akochan.

[demo](https://gh.ekyu.moe/akochan-reviewer-demo.html)

**This tool is still in early stages. There are still lots of features to be implemented, and breaking changes may be made at anytime. Suggestions and contributions are welcome. 日本語もおｋ.**

## Usage
```plain
USAGE:
    akochan-reviewer [FLAGS] [OPTIONS] --actor <INDEX>

FLAGS:
    -f, --full              Analyze every move, not only the different ones.
    -h, --help              Prints help information
        --no-open           Do not open the output file after finishing
        --no-review         Do not review at all. Only download and save files
    -V, --version           Prints version information
    -v, --verbose           Use verbose output
        --without-viewer    Do not include log viewer in the generated HTML report

OPTIONS:
    -a, --actor <INDEX>            Specify the actor to review. It is the number after "&tw=" in tenhou's log url
    -d, --akochan-dir <DIR>        Specify the directory of akochan. This will serves as the working directory of
                                   akochan process. Default value is the directory in which --akochan-exe is specified
    -e, --akochan-exe <EXE>        Specify the executable file of akochan. Default value "akochan/system.exe"
    -i, --in-file <FILE>           Specify a tenhou.net/6 format log file to review. If FILE is "-" or empty, read from
                                   stdin
        --mjai-out <FILE>          Save the transformed mjai format log to FILE. If FILE is "-", write to stdout
    -o, --out-file <FILE>          Specify the output file for generated HTML report. If FILE is "-", write to stdout;
                                   if FILE is empty, write to "{tenhou_id}&tw={actor}.html" if --tenhou-id is specified,
                                   otherwise "report.html"
        --pt <ARRAY>               Shortcut to override "jun_pt" in --tactics-config. Format: "90,45,0,-135"
    -c, --tactics-config <FILE>    Specify the tactics config file for akochan. Default value "tactics.json"
    -t, --tenhou-id <ID>           Specify a Tenhou log ID to review, overriding --in-file. Example: "2019050417gm-0029-
                                   0000-4f2a8622"
        --tenhou-out <FILE>        Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is specified. If
                                   FILE is "-", write to stdout
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

## Acknowledgment
* [critter](https://twitter.com/critter_Eng): The creater of akochan, who also proposed many advise and gave help to the development of akochan-reviewer.

## License
[Apache-2.0](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)
