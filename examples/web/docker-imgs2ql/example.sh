#!/bin/sh

export LISTEN_ADDR=127.0.0.1:7229
export DOCKER_SOCK_PATH=./remote-docker.sock
export DOCKER_TIMEOUT_SECONDS=60

echo listen addr: ${LISTEN_ADDR}

./docker-imgs2ql
