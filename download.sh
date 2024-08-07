#!/bin/bash

UNAME_CMD="$(uname -m)"
case "$UNAME_CMD" in
x86_64) SUFFIX="x86_64" ;;
aarch64) SUFFIX="aarch64" ;;
armv6l) SUFFIX="armv6" ;;
esac

if [ -n "$SUFFIX" ]; then
	SCREEN_CTL="screen_control_linux_${SUFFIX}.tar.gz"
	curk -L -O "https://github.com/mrjackwills/screen_control_backend/releases/latest/download/${SCREEN_CTL}"
	tar xzvf "${SCREEN_CTL}" screen_control
	rm "${SCREEN_CTL}"
fi
