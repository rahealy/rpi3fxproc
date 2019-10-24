#
# Makefile

# Targets:
#
#  all - Builds debug version.
#  release - Builds optimized release version.
#  clean - Cleans the build directories
#
# Example:
#  $ make clean release
#

#######################################################################
# Variables
#######################################################################

#Build
TARGET = aarch64-unknown-none
KERNEL = kernel8.img

#Bootloader
TX_PATH = ../rpi3serbtldr/tx
TX_EXE  = $(TX_PATH)/rpi3serbtldr_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV)

#######################################################################
# Targets
#######################################################################

all: debug

debug:
	$(MAKE) -C ./rpi3serbtldr debug
	$(MAKE) -C ./rpibmtkr debug

release:
	$(MAKE) -C ./rpi3serbtldr release
	$(MAKE) -C ./rpibmtkr release

clean:
	cargo clean
	$(MAKE) -C ./rpi3serbtldr clean
	$(MAKE) -C ./rpibmtkr clean
