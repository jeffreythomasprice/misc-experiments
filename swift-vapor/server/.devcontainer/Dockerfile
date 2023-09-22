FROM swift:5.8.1

ARG USERNAME=user
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME && \
	useradd --uid $USER_UID --gid $USER_GID -m $USERNAME && \
	apt-get update && \
	apt-get install -y sudo && \
	echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
	chmod 0440 /etc/sudoers.d/$USERNAME

RUN apt update
RUN apt install -y build-essential curl

ENV NVM_DIR /usr/local/nvm
ENV NODE_VERSION 20.7.0
RUN mkdir -p $NVM_DIR && \
	curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash && \
	. $NVM_DIR/nvm.sh && \
	nvm install $NODE_VERSION && \
	nvm alias default $NODE_VERSION && \
	nvm use default
ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/v$NODE_VERSION/bin:$PATH

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

USER $USERNAME

WORKDIR /
