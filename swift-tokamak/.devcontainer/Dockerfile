FROM ghcr.io/swiftwasm/carton:latest

ARG USERNAME=user
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME && \
	useradd --uid $USER_UID --gid $USER_GID -m $USERNAME && \
	apt-get update && \
	apt-get install -y sudo && \
	echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
	chmod 0440 /etc/sudoers.d/$USERNAME

RUN apt update && \
	apt install -y libgtk-3-dev

USER $USERNAME

RUN carton sdk install wasm-5.9-SNAPSHOT-2023-08-01-a
ENV PATH "/home/user/.carton/sdk/wasm-5.9-SNAPSHOT-2023-08-01-a/usr/bin:$PATH"

WORKDIR /home/user
RUN git clone https://github.com/apple/swift-format.git
WORKDIR /home/user/swift-format
RUN git checkout "tags/509.0.0"
RUN swift build -c release
ENV PATH "/home/user/swift-format/.build/release:$PATH"

WORKDIR /workspaces
