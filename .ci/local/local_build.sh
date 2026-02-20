#!/usr/bin/env bash

source .ci/local/generate_pkgbuild.sh

rm -rf target/pkgbuild/local
mkdir -p target/pkgbuild/local

tar -vzcf target/pkgbuild/local/${REL_NAME}.tar.gz --transform "s/^.\//${REL_NAME}\//" ./src ./conf ./Cargo.*

generate_pkgbuild local target/pkgbuild/local/PKGBUILD

makepkg -si -D target/pkgbuild/local -f
