# akochan-reviewer

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Equim-chan/akochan-reviewer/Rust)](https://github.com/Equim-chan/akochan-reviewer/actions)
[![LICENSE](https://img.shields.io/github/license/Equim-chan/akochan-reviewer.svg)](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)

Review your Tenhou log with mahjong AI akochan.

[Demo](https://gh.ekyu.moe/akochan-reviewer-demo.html)

## Build
First of all, build [akochan of my fork](https://github.com/Equim-chan/akochan).

```console
$ git clone https://github.com/Equim-chan/akochan.git
$ cd akochan
```

You have to edit `Makefile` and `ai_src/Makfefile` accordingly. Set up correct path for boost and some other options like `-march=native` of your choice.

```console
$ cd ai_src
$ make ai.dll # or libai.so
$ cd ..
$ make system.exe
```

Then, build akochan-reviewer

```console
$ cd ..
$ git clone https://github.com/Equim-chan/akochan-reviewer.git
$ export RUSTFLAGS="-C target-cpu=native" # optional
$ cargo build --release
```

`akochan-reviewer` binary will be in `target/release` directory.

## Usage
```plain
USAGE:
    akochan-reviewer [FLAGS] [OPTIONS] --actor <INDEX>

FLAGS:
    -h, --help              Prints help information
        --no-open           Do not open the output file after finishing
        --no-review         Do not review at all. Only download and save files
    -V, --version           Prints version information
        --without-viewer    Do not include log viewer in the generated HTML report

OPTIONS:
    -a, --actor <INDEX>            Specify the actor to review
    -d, --akochan-dir <DIR>        Specify the directory of akochan. This will serves as the working directory of
                                   akochan process. Default value is the directory in which --akochan-exe is specified
    -e, --akochan-exe <EXE>        Specify the executable file of akochan. Default value "akochan/system.exe"
    -i, --in-file <FILE>           Specify a tenhou.net/6 format log file to review. If FILE is "-" or empty, read from
                                   stdin
        --mjai-out <FILE>          Save the transformed mjai format log to FILE. If FILE is "-", write to stdout
    -o, --out-file <FILE>          Specify the output file for generated HTML report. If FILE is "-", write to stdout;
                                   if FILE is empty, write to "{tenhou_id}&tw={actor}.html" if --tenhou-id is specified,
                                   otherwise "report.html"
    -c, --tactics-config <FILE>    Specify the tactics config file for akochan. Default value "tactics.json"
    -t, --tenhou-id <ID>           Specify a Tenhou log ID to review, overriding --in-file. For example: 2019050417gm-
                                   0029-0000-4f2a8622
        --tenhou-out <FILE>        Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is specified. If
                                   FILE is "-", write to stdout
```

## Acknowledgment
* [critter](https://twitter.com/critter_Eng): The creater of akochan, who also proposed many advise and gave help to the development of akochan-reviewer.

## License
[Apache-2.0](https://github.com/Equim-chan/akochan-reviewer/blob/master/LICENSE)
