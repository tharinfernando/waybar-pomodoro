BINARY      := waybar-pomodoro
INSTALL_DIR := $(HOME)/.local/bin
STATE_DIR   := $(HOME)/.local/share/waybar-pomodoro

.PHONY: all build install uninstall

all: build

build:
	cargo build --release

install: build
	@mkdir -p $(INSTALL_DIR)
	cp target/release/$(BINARY) $(INSTALL_DIR)/
	@echo "Patching Waybar config..."
	@python3 scripts/patch-waybar.py install
	@echo "Restarting Waybar..."
	@command -v omarchy-restart-waybar >/dev/null 2>&1 \
		&& omarchy-restart-waybar \
		|| (killall waybar 2>/dev/null; waybar &)
	@echo "Done. $(BINARY) is now live in Waybar."

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY)
	rm -rf $(STATE_DIR)
	@echo "Patching Waybar config..."
	@python3 scripts/patch-waybar.py uninstall
	@echo "Restarting Waybar..."
	@command -v omarchy-restart-waybar >/dev/null 2>&1 \
		&& omarchy-restart-waybar \
		|| (killall waybar 2>/dev/null; waybar &)
	@echo "Done. $(BINARY) has been removed."
