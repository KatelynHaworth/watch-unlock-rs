# Maintainer: Katelyn 'KatLongLegs' Haworth <katelyn+github@haworth.id.au>
pkgname=watch-unlock-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="A PAM module, and CLI, that provides auth using an Apple Watch"
arch=("x86_64")
url="https://github.com/KatelynHaworth/${pkgname}"
license=('MIT')
depends=("bluez" "glibc" "pam")
makedepends=("cargo")
options=("!strip")
source=(
    $pkgname-$pkgver.tar.gz::${url}/archive/v$pkgver.tar.gz
    $pkgname-$pkgver.tar.gz.sig::${url}/releases/download/v$pkgver/v$pkgver.tar.gz.sig
)
sha256sums=(
    "e15364f492395019dd6ffdf8c2de35177f84b87c3c121e0e1e12b5027bba22c0" # tarball
    "982b1a291ed43068263f034c7649e12863f3b490b4cbd1327962eca368410ad6" # signature
)
validpgpkeys=(
    843A4317E6A6933E196CFEF7940D6D15EAB71277 # Katelyn 'KatLongLegs' Haworth
)

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
    cargo build --frozen --release --lib
    mv target/release/libpam_apple_watch.so target/release/pam_apple_watch.so
}

package() {
	install -Dm0755 -t "${pkgdir}/usr/bin/" "target/release/watch_unlock_cli"
	install -Dm0755 -t "${pkgdir}/usr/lib/security/" "target/release/pam_apple_watch.so"
	install -Dm0644 -t "${pkgdir}/etc/security/" "${startdir}/conf/security/apple_watch.conf"
	install -Dm0644 -t "${pkgdir}/etc/pam.d/" "${startdir}/conf/pam.d/apple-watch"
}
