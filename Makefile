PREFIX = /usr/local

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
	@mkdir -p ${PREFIX}/bin
	@cp -f target/release/penrose ${PREFIX}/bin
	@cp -f bin/lock-screen ${PREFIX}/bin
	@cp -f bin/rofi-apps ${PREFIX}/bin
	@cp -f bin/run-penrose ${PREFIX}/bin
	@chmod 755 ${PREFIX}/bin/penrose
	@chmod 755 ${PREFIX}/bin/lock-screen
	@chmod 755 ${PREFIX}/bin/rofi-apps
	@chmod 755 ${PREFIX}/bin/run-penrose
	@echo ":: Installing utility scripts..."
	@cp -r scripts ${PREFIX}
	@ls scripts | grep -v lock.png | xargs -I {} chmod 755 ${PREFIX}/scripts/{}
	@echo ":: Done"

.PHONY: uninstall
uninstall:
	@echo ":: Removing binaries..."
	@rm -f ${PREFIX}/bin/penrose\
		${PREFIX}/bin/lock-screen\
		${PREFIX}/bin/rofi-apps\
		${PREFIX}/bin/run-penrose
	@echo ":: Removing scripts..."
	@ls scripts | xargs -I {} rm -f ${PREFIX}/scripts/{}
	@echo ":: Done"
