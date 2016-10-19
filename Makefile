# Toolchain
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump

# Target
TARGET=thumbv7em-none-eabi

# Files
OUT_DIR=target/$(TARGET)/release
OUT_FILE=$(OUT_DIR)/zinc_cube

.PHONY: build clean listing $(OUT_FILE)

all: build listing
build: $(OUT_FILE).bin
listing: $(OUT_FILE).lst

$(OUT_FILE):
	cargo build --release --target=$(TARGET)

$(OUT_DIR)/%.bin: $(OUT_DIR)/%
	$(OBJCOPY) -O binary $< $@

$(OUT_DIR)/%.lst: $(OUT_DIR)/%
	$(OBJDUMP) -D $< > $@

clean:
	cargo clean
