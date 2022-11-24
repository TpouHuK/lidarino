#!/bin/bash
cross build && scp ./target/arm-unknown-linux-gnueabihf/debug/lidarino pi@raspberrypi.local:~/
