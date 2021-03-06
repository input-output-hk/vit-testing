# Simple dockerfile example to build a jormungandr voting backend 

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Jormungandr Disruption backend"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}

# prepare the environment
RUN apt-get update && \
    apt-get install -y git curl && \
    mkdir -p ${ENV_PREFIX}

#install rustup
RUN  apt-get install -y build-essential pkg-config libssl-dev && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
     ~/.cargo/bin/rustup install stable && \
    ~/.cargo/bin/rustup default stable

RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/jormungandr && \
	cd jormungandr && \
	git checkout cbd280b4652bd397c34b25caff28d816c6be3941

RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/vit-servicing-station && \
	cd vit-servicing-station && \
	git checkout df9490ae33bb3abef8cb6877001091b179c5d38b
	
 RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/vit-testing

# install the node and jcli from source
RUN cd ${ENV_PREFIX}/vit-servicing-station && \
    ~/.cargo/bin/cargo install --locked --force --path vit-servicing-station-server
    
# install the node and jcli from source
RUN cd ${ENV_PREFIX}/jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jcli

# install the node and jcli from source
RUN cd ${ENV_PREFIX}/vit-testing && \
    ~/.cargo/bin/cargo install --locked --path vitup && \
    ~/.cargo/bin/cargo install --locked --path iapyx

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 80 8000 8001 8002 8003 8004 8005 8006 8007 8008 8009 8010 3030

WORKDIR ${ENV_PREFIX}/vit-testing/vitup
RUN mkdir -p ./mock

ENTRYPOINT ~/.cargo/bin/vitup start mock --config ./example/mock/config.yaml --params ./example/mock/start_params.yaml
