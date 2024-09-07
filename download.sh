#!/bin/bash

UNAME_CMD="$(uname -m)"
case "$UNAME_CMD" in
x86_64) SUFFIX="x86_64" ;;
aarch64) SUFFIX="aarch64" ;;
armv6l) SUFFIX="armv6" ;;
esac



# screen_control_linux_aarch64.tar.gz 

if [ -n "$SUFFIX" ]; then
	SCREEN_CTL="screen_control_linux_${SUFFIX}.tar.gz"
	curl -L -O "https://github.com/mrjackwills/screen_control_backend/releases/latest/download/${SCREEN_CTL}"
	tar xzvf "${SCREEN_CTL}" screen_control
	rm "${SCREEN_CTL}"
fi
