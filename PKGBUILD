# Maintainer: Katelyn 'KatLongLegs' Haworth <katelyn+github@haworth.id.au>
pkgname=watch-unlock-rs
pkgver=0.2.0
pkgrel=0
pkgdesc="A PAM module, and CLI, that provides auth using an Apple Watch"
arch=("x86_64")
url="https://github.com/KatelynHaworth/${pkgname}"
license=('MIT')
depends=("bluez" "glibc" "pam")
makedepends=("cargo")
options=("!strip")
source=(
  ${url}/releases/download/v$pkgver/$pkgname-$pkgver.tar.gz{,.sig}
)
sha256sums=(
  1c86ab2e51c4dfcefbc26e20648d3a074b9619ab9f02f17e6b5596066697496c
  c103abae9739ff130091dd4fa3669d263b0bb41f7ef89b65432c4ffd62bb48d8
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
  install -Dm0644 -t "${pkgdir}/etc/security/" "${pkgname}-${pkgver}/conf/security/apple_watch.conf"
  install -Dm0644 -t "${pkgdir}/etc/pam.d/" "${pkgname}-${pkgver}/conf/pam.d/apple-watch"
}
