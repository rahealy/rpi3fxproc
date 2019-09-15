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

BINARY = rpi3serbtldr_tx

all: debug

debug:
	$(MAKE) -C rx
	$(MAKE) -C tx

release:
	$(MAKE) -C rx release
	$(MAKE) -C tx release

clean:
	$(MAKE) -C rx clean
	$(MAKE) -C tx clean
