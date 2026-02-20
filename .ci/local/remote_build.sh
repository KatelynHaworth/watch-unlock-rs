#!/usr/bin/env bash

rm -rf target/pkgbuild/remote
mkdir -p target/pkgbuild/remote

cp PKGBUILD target/pkgbuild/remote/PKGBUILD

makepkg -si -D target/pkgbuild/remote -f
