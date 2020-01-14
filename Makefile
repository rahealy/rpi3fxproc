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
NAME = rpi3fxproc
TARGET = aarch64-unknown-none-softfloat
TARGET = aarch64-unknown-none
KERNEL = kernel8
KERNEL_IMAGE = kernel8.img

#Bootloader
TX_PATH = ./bootloader/tx
TX_EXE  = $(TX_PATH)/bootloader_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV) -e


#######################################################################
# Targets
#######################################################################

all: none

debug: none_debug

softfloat:
	cargo xrustc --target=$(TARGET_SOFTFLOAT) --release -- --verbose --emit asm
	cp ./target/$(TARGET_SOFTFLOAT)/release/$(NAME) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

none:
	cargo xrustc --target=$(TARGET_NONE) --release -- --verbose --emit asm
	cp ./target/$(TARGET_NONE)/release/$(NAME) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

softfloat_debug:
	cargo xrustc --target=$(TARGET_SOFTFLOAT) -- --verbose --emit asm 
	cp ./target/$(TARGET_SOFTFLOAT)/debug/$(NAME) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

none_debug:
	cargo xrustc --target=$(TARGET_NONE) -- --verbose --emit asm
	cp ./target/$(TARGET_NONE)/debug/$(NAME) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

clean:
	cargo clean
	-rm -f ./$(KERNEL)
	-rm -f ./$(KERNEL_IMAGE)

realclean: clean
	$(MAKE) -C ./bootloader clean
	$(MAKE) -C ./common clean
	$(MAKE) -C ./hardware clean
	$(MAKE) -C ./i2squeue clean
	$(MAKE) -C ./rack clean

_bootloader:
	$(MAKE) -C ./bootloader release

load: _bootloader
	$(TX_EXE) $(TX_ARGS) -f ./$(KERNEL_IMAGE)

#######################################################################
# Experimental Targets
#######################################################################

CONTAINER_UTILS   = andrerichter/raspi3-utils

DOCKER_CMD        = docker run -p 1234:1234 -it --rm
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work
DOCKER_ARG_TTY    = --privileged -v /dev:/dev
DOCKER_EXEC_QEMU  = qemu-system-aarch64 -s -S -M raspi3 -kernel kernel8.img

TARGET_NONE = aarch64-unknown-none
TARGET_SOFTFLOAT = aarch64-unknown-none-softfloat

qemu:
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_QEMU) -serial stdio

objdump:
	cargo objdump --target $(TARGET_NONE) -- -disassemble -print-imm-hex $(KERNEL) > list.lst

objdump_none: none
	cargo objdump --target $(TARGET_NONE) -- -disassemble -print-imm-hex $(KERNEL) > none.lst

objdump_softfloat: softfloat
	cargo objdump --target $(TARGET_SOFTFLOAT) -- -disassemble -print-imm-hex $(KERNEL) > softfloat.lst
