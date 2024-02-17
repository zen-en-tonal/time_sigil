FROM rust:1.76-buster

WORKDIR /workspace

SHELL [ "/bin/bash", "-c" ]

RUN apt update && \
    apt install git && \
    apt clean
RUN groupadd vscode && \
    useradd -d /home/vscode -s /bin/bash -g vscode -m vscode

USER vscode

RUN rustup component add rustfmt
