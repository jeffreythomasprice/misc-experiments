FROM swift:5.8.1

RUN apt update
RUN apt install -y build-essential

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

WORKDIR /
