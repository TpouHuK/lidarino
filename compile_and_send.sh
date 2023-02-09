#!/bin/bash

# TODO, add propper flags handling
# right now usage is
# `./compile_and_send.sh BINARY_NAME [--release]`
# --release flag must be after BINARY_NAME
# BINARY_NAME is mandatory
#
# Examples:
# `./compile_and_send.sh lidarino_cli --release`
# `./compile_and_send.sh http_server `

BINARY=$1
DESTINATION="pi@raspberrypi.local:~/"
#DESTINATION="pi@10.13.202.3:~/"

if [[ $* == *--release* ]]
then
	MODE="--release"
	DIR="release"
else
	MODE=""
	DIR="debug"
fi

cross build -v --target=arm-unknown-linux-gnueabihf $MODE --bin $BINARY && scp ./target/arm-unknown-linux-gnueabihf/$DIR/$BINARY $DESTINATION
