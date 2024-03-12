CC = gcc
CFLAGS = -Wall -Werror
TARGET = power_seek
BUILD_DIR = build
SRC_DIR = src

SRC = $(wildcard $(SRC_DIR)/*.c)

all: $(TARGET)

# a build dir is created to store the object files
$(BUILD_DIR)/:
	mkdir -p $(BUILD_DIR)

$(TARGET): $(BUILD_DIR)/
	$(CC) $(CFLAGS) -o $(BUILD_DIR)/$(TARGET) $(SRC)

run: $(TARGET)
	./$(BUILD_DIR)/$(TARGET)

watch: $(TARGET)
	watch -n 0.5 ./$(BUILD_DIR)/$(TARGET)

gdb: $(TARGET)
	gdb $(BUILD_DIR)/$(TARGET)

clean:
	rm -rf $(BUILD_DIR)

deploy: $(TARGET)
	cp $(BUILD_DIR)/$(TARGET) /usr/local/bin/$(TARGET)

remove:
	rm -f /usr/local/bin/$(TARGET)