# Maintainer: Katelyn 'KatLongLegs' Haworth <katelyn+github@haworth.id.au>
pkgname=watch-unlock-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="A PAM module, and CLI, that provides auth using an Apple Watch"
arch=("x86_64")
url="https://github.com/KatelynHaworth/watch-unlock-rs"
license=('MIT')
groups=()
depends=("bluez" "glibc" "pam")
makedepends=("cargo")
options=("!strip")
source=("$pkgname-$pkgver.tar.gz::https://github.com/KatelynHaworth/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=("e53c6ad449f26c6610f1127ba77d60105a7001c135f29e89be31911f7d70edd0")
validpgpkeys=()

prepare() {
	export RUSTUP_TOOLCHAIN=stable

	echo "Fetching dependencies..."
    cargo fetch --locked --target host-tuple
}

build() {
	export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target

    echo "Building CLI tool..."
    cargo build --frozen --release --bin watch_unlock_cli --features="cli"

    echo "Building PAM module..."
    cargo build --frozen --release --lib --features="lib"
    mv target/release/libpam_apple_watch.so target/release/pam_apple_watch.so
}

package() {
	install --debug -Dm0755 -t "${pkgdir}/usr/bin/" "target/release/watch_unlock_cli"
	install --debug -Dm0755 -t "${pkgdir}/usr/lib/security/" "target/release/pam_apple_watch.so"
	install --debug -Dm0644 -t "${pkgdir}/etc/security/" "${startdir}/conf/security/apple_watch.conf"
	install --debug -Dm0644 -t "${pkgdir}/etc/pam.d/" "${startdir}/conf/pam.d/apple-watch"
}
