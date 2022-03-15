FROM rust:1.59.0


# install akochan deps
RUN set -ex \
  && apt-get update && apt-get install -y \
    libboost-all-dev \
  && apt-get clean && rm -rf /var/lib/apt/lists/*


# build akochan ai and system
# ref: https://github.com/Equim-chan/akochan#systemexe%E3%82%B3%E3%83%B3%E3%83%91%E3%82%A4%E3%83%AB%E6%89%8B%E9%A0%86linux
WORKDIR /akochan-reviewer

COPY akochan akochan

WORKDIR /akochan-reviewer/akochan/ai_src

RUN set -ex \
  && make -f Makefile_Linux libai.so \
  && cp libai.so ../

WORKDIR /akochan-reviewer/akochan

RUN set -ex \
  && make -f Makefile_Linux system.exe

# set path for system.exe to find libai.so
ENV LD_LIBRARY_PATH $LD_LIBRARY_PATH:/akochan-reviewer/akochan


# build akochan-reviewer
# ref: https://github.com/Equim-chan/akochan-reviewer#build
WORKDIR /akochan-reviewer

COPY . .

RUN set -ex \
  && cargo build --release


ENTRYPOINT ["./target/release/akochan-reviewer"]
