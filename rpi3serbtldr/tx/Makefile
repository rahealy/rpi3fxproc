#
# Makefile
#  Rust cargo doesn't quite have the functionality we need yet so using
#  a very simple makefile to automate building.
#
# Targets:
#
#  all - Builds debug version.
#  release - Builds optimized release version.
#  clean - Cleans the build directories
#
# Example:
#  $ make clean release
#
ROOT_DIR:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
BINARY = rpi3serbtldr_tx

all: debug

debug:
	cargo build
	cp $(ROOT_DIR)/target/debug/$(BINARY) .

release:
	cargo build --release
	cp $(ROOT_DIR)/target/release/$(BINARY) .

clean:
	cargo clean
	-rm $(BINARY)
