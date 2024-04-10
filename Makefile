.RECIPEPREFIX = >

# Variables
PROJECT_NAME = term-typist
RELEASE_BINARY = target/release/$(PROJECT_NAME)
INSTALL_DIR = /usr/bin/
CONFIG_DIR = /home/$(USER)/.config/term-typist
LOCAL_DIR = /home/$(USER)/.local/share/term-typist

# Default target
all: build

# Build target
build:
> mkdir -p $(CONFIG_DIR)
> mkdir -p $(LOCAL_DIR)
> cp -r words $(LOCAL_DIR)
> cargo build --release

# Install target
install:
> cp $(RELEASE_BINARY) $(INSTALL_DIR)

# Uninstall target
uninstall:
> rm -f $(INSTALL_DIR)$(PROJECT_NAME)

# Clean target
clean:
> cargo clean

.PHONY: all build install uninstall clean
