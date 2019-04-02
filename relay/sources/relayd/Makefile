DESTDIR = $(CURDIR)/target

# fmt, build, test, audit, clean, bench, etc.
%:
	cargo $@

outdated:
	# only check on our dependencies
	cargo outdated --root-deps-only

lint:
	RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features --examples --tests -- -D clippy::all

install: clean lint audit
	cargo install --bin relayd --path=. --root=$(DESTDIR)

yum-dependencies:
	yum install pkgconf-pkg-config libpqxx openssl-devel zlib-devel

apt-dependencies:
	apt install pkg-config libpq-dev libssl-dev zlib1g-dev

build-env:
	curl https://sh.rustup.rs -sSf | sh
	rustup component add clippy
	cargo install cargo-update
	cargo install cargo-audit

build-env-update:
	rustup self update
	rustup update
	cargo install-update -a

deps-update: update outdated;

dev-env: build-env
	rustup component add rustfmt
	cargo install cargo-outdated

veryclean: clean
	rustup self uninstall
	rm -rf ~/.rustup ~/.cargo
