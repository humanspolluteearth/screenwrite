# Maintainer: your-name <your@email.com>
pkgname=focus-write
pkgver=0.1.0
pkgrel=1
pkgdesc="Frameless minimalist GPU-accelerated writing environment for Arch Linux"
arch=('x86_64')
url="https://github.com/your-user/focus-write"
license=('MIT')
depends=(
    'fontconfig'
    'libxkbcommon'
    'vulkan-icd-loader'
    'ttf-jetbrains-mono'
)
makedepends=(
    'rust'
    'cargo'
)
source=("$pkgname::git+file://$PWD")
sha256sums=('SKIP')

build() {
    cd "$pkgname"
    cargo build --release --locked
}

check() {
    cd "$pkgname"
    cargo test
}

package() {
    cd "$pkgname"
    install -Dm755 "target/release/focus-write" "$pkgdir/usr/bin/focus-write"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
