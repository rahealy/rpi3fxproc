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

clean:
	$(MAKE) -C ./effects clean 
	cargo clean
