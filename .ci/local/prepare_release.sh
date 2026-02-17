#!/usr/bin/env bash

NAME="watch-unlock-rs"
VERSION=$1
echo "Preparing release for v${VERSION}"

echo "Creating GIT tag for release version"
git tag --sign -a v${VERSION}

mkdir -p target/release/gh_release
echo "Creating release tarball"
RELEASE="${NAME}-${VERSION}"
TAR_FILE="target/release/gh_release/v${VERSION}.tar.gz"
git -c tar.tar.gz.command='gzip -cn' archive --format=tar.gz --prefix="${RELEASE}/" -o "${TAR_FILE}" "v${VERSION}"

echo "Signing release tarball"
TAR_SIG_FILE="target/release/gh_release/v${VERSION}.tar.gz.sig"
gpg --detach-sign --output "${TAR_SIG_FILE}" target/release/gh_release/v${VERSION}.tar.gz

echo "Updating PKGBUILD"
cp .ci/local/PKGBUILD.source ./PKGBUILD
sed -i "s/{{VERSION}}/${VERSION}/" PKGBUILD
sed -i "s/{{TAR_SHASUM}}/$(sha256sum ${TAR_FILE} | awk '{print $1}')/" PKGBUILD
sed -i "s/{{TAR_SIG_SHASUM}}/$(sha256sum ${TAR_SIG_FILE} | awk '{print $1}')/" PKGBUILD