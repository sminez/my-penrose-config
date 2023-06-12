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
	@chmod 755 /usr/local/bin/penrose
	@cp -R bin/. /usr/local/bin
	@ls bin | xargs -I {} chmod 755 /usr/local/bin/{}
	@echo ":: Installing utility scripts..."
	@cp -r scripts /usr/local
	@ls scripts | grep -v lock.png | xargs -I {} chmod 755 /usr/local/scripts/{}
	@echo ":: Done"

.PHONY: uninstall
uninstall:
	@echo ":: Removing binaries..."
	@ls bin | xargs -I {} rm -f /usr/local/bin/{}
	@rm -f /usr/local/bin/penrose
	@echo ":: Removing scripts..."
	@ls scripts | xargs -I {} rm -f /usr/local/scripts/{}
	@echo ":: Done"
