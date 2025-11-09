FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
	build-essential \
	cmake \
	git \
	curl \
	wget \
	clang-15 \
	llvm-15 \
	llvm-15-dev \
	pkg-config \
	libssl-dev \
	python3 \
	python3-pip \
	ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

ENV LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
ENV PATH="$LLVM_SYS_150_PREFIX/bin:$PATH"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /workspace

CMD ["/bin/bash"]
