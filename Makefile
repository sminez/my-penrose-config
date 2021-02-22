# TODO: manpages?
#       other binaries

all: build

.PHONY: build
build:
	$(shell [[ $EUID -eq 0 ]] && echo "build can not be run as root" && exit 1)
	@echo ":: Rebuilding in release mode..."
	@cargo build --release

.PHONY: install
install:
	@echo ":: Installing binaries..."
	@mkdir -p /usr/local/bin
	@cp -f target/release/penrose /usr/local/bin
	@cp -f target/release/pmenu /usr/local/bin
	@cp -f bin/lock-screen /usr/local/bin
	@cp -f bin/p_log /usr/local/bin
	@cp -f bin/rofi-apps /usr/local/bin
	@cp -f bin/run-penrose /usr/local/bin
	@chmod 755 /usr/local/bin/penrose
	@chmod 755 /usr/local/bin/pmenu
	@chmod 755 /usr/local/bin/lock-screen
	@chmod 755 /usr/local/bin/p_log
	@chmod 755 /usr/local/bin/rofi-apps
	@chmod 755 /usr/local/bin/run-penrose
	@echo ":: Installing utility scripts..."
	@cp -r scripts /usr/local
	@ls scripts | grep -v lock.png | xargs -I {} chmod 755 /usr/local/scripts/{}
	@echo ":: Done"

.PHONY: uninstall
uninstall:
	@echo ":: Removing binaries..."
	@rm -f /usr/local/bin/penrose\
		/usr/local/bin/pmenu\
		/usr/local/bin/lock-screen\
		/usr/local/bin/p_log\
		/usr/local/bin/rofi-apps\
		/usr/local/bin/run-penrose
	@echo ":: Removing scripts..."
	@ls scripts | xargs -I {} rm -f /usr/local/scripts/{}
	@echo ":: Done"
