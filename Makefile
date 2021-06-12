SERVICE := merklechat
DESTDIR ?= dist_root
SERVICEDIR ?= /srv/$(SERVICE)

.PHONY: build install build-server build-wasm

build: build-server build-wasm build-avatars build-checker


build-avatars:
	# $(MAKE) -C www/img


build-wasm:
	CARGO_HOME=$(CURDIR)/.cargo cargo install wasm-pack
	(cd wasm ; PATH=$(PATH):$(CURDIR)/.cargo/bin CARGO_HOME=$(CURDIR)/.cargo wasm-pack build --target web --release)
	zopfli wasm/pkg/*.wasm


build-server:
	(cd server ; CARGO_HOME=$(CURDIR)/.cargo cargo build --release)


build-checker:
	(cd checker ; CARGO_HOME=$(CURDIR)/.cargo cargo build --release)


install: build
	mkdir -p $(DESTDIR)$(SERVICEDIR)
	install -m 555 server/target/release/merkleserver $(DESTDIR)$(SERVICEDIR)/
	install -d $(DESTDIR)$(SERVICEDIR).www/wasm
	install -d $(DESTDIR)$(SERVICEDIR).www/css
	install -d $(DESTDIR)$(SERVICEDIR).www/img
	install -m 444 wasm/pkg/*    $(DESTDIR)$(SERVICEDIR).www/wasm/
	install -m 444 www/*.html    $(DESTDIR)$(SERVICEDIR).www/
	install -m 444 www/css/*.css $(DESTDIR)$(SERVICEDIR).www/css/
	install -m 444 www/img/*.svg $(DESTDIR)$(SERVICEDIR).www/img/

	install -d $(DESTDIR)/etc/nginx/sites-enabled/
	install -m 444 etc/nginx/sites-enabled/merklechat.conf $(DESTDIR)/etc/nginx/sites-enabled/
	mkdir -p $(DESTDIR)/etc/systemd/system
	install -m 444 server/merklechat.service $(DESTDIR)/etc/systemd/system
	install -m 444 server/system-merklechat.slice $(DESTDIR)/etc/systemd/system
