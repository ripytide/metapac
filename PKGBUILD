# Maintainer: James Forster <james.forsterer@gmail.com>

pkgname=metapac-bin
pkgver=0.2.0
pkgrel=1
pkgdesc="multi-backend declarative package manager"
url="https://github.com/ripytide/metapac"
license=("GPL-3.0-or-later")
arch=("x86_64")
provides=("metapac")
conflicts=("metapac")
source=("https://github.com/ripytide/metapac/releases/download/v$pkgver/metapac-x86_64-unknown-linux-gnu.tar.xz")
sha256sums=("75b73137f35ba659f3a8583f3384d015c4f3d659a2d056841404870023bd56a6")

package() {
    install -Dm755 metapac -t "$pkgdir/usr/bin"
}
