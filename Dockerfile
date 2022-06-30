# This Dockerfile only has akochan engine embedded.

FROM rust:1.62.0

# install akochan deps
RUN set -ex \
  && apt-get update && apt-get install -y \
    libboost-all-dev \
  && apt-get clean && rm -rf /var/lib/apt/lists/*


# build akochan ai and system
# ref: https://github.com/Equim-chan/akochan#systemexe%E3%82%B3%E3%83%B3%E3%83%91%E3%82%A4%E3%83%AB%E6%89%8B%E9%A0%86linux
WORKDIR /mjai-reviewer

COPY akochan akochan

WORKDIR /mjai-reviewer/akochan/ai_src

RUN set -ex \
  && make -f Makefile_Linux libai.so \
  && cp libai.so ../

WORKDIR /mjai-reviewer/akochan

RUN set -ex \
  && make -f Makefile_Linux system.exe

# set path for system.exe to find libai.so
ENV LD_LIBRARY_PATH $LD_LIBRARY_PATH:/mjai-reviewer/akochan


# build mjai-reviewer
# ref: https://github.com/Equim-chan/mjai-reviewer#build
WORKDIR /mjai-reviewer

COPY . .

RUN set -ex \
  && cargo build --release


ENTRYPOINT ["./target/release/mjai-reviewer"]
