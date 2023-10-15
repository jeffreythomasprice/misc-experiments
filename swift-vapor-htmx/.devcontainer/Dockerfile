FROM swift:5.9.0

# setup

ARG USERNAME=user
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME && \
	useradd --uid $USER_UID --gid $USER_GID -m $USERNAME && \
	apt-get update && \
	apt-get install -y sudo && \
	echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
	chmod 0440 /etc/sudoers.d/$USERNAME

# build deps

RUN apt update && \
	apt install -y \
		build-essential \
		curl \
		wget

WORKDIR /watchexec
RUN wget https://github.com/watchexec/watchexec/releases/download/v1.23.0/watchexec-1.23.0-x86_64-unknown-linux-gnu.deb && \
	dpkg -i watchexec-1.23.0-x86_64-unknown-linux-gnu.deb

WORKDIR /
RUN git clone https://github.com/apple/swift-format.git
WORKDIR /swift-format
RUN git checkout "tags/509.0.0"
RUN swift build -c release
RUN cp .build/release/swift-format /usr/local/bin/

WORKDIR /
RUN git clone https://github.com/vapor/toolbox.git
WORKDIR /toolbox
RUN git checkout 18.7.1
RUN make install

# run deps

RUN apt install -y \
	libsqlite3-dev

# done

USER $USERNAME

WORKDIR /workspace
