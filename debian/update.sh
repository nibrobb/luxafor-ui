#!/usr/bin/env bash

set -x

[[ -n $GPG_SIGNING_KEY && -n $GPG_SIGNING_EMAIL ]] || exit 1

# sudo apt update
# sudo apt install -y wget git gpg apt-utils dpkg-dev
# git checkout v${{ github.event.inputs.version }}
# cd ${{ github.workspace }}
# gh release download debian -D debian -R nibrobb/luxafor-ui
# mv -v ./*.deb ./debian # hmm
# cd debian

# Get the OS code-name ("noble" on Ubuntu 24.04 LTS)
codename="$(lsb_release -sc 2>/dev/null)"

deb_arch=$(dpkg --print-architecture)

cat <<EOF > Release
Origin: luxafor-ui
Label: nibrobb luxafor-ui
Suite: $codename
Codename: $codename
Architectures: $deb_arch
Components: main
Description: Graphical desktop application for controlling a Luxafor FLAG™
EOF

cp ../target/release/bundle/deb/*.deb ./
# gh release download latest -R nibrobb/luxafor-ui # or something

apt-ftparchive --arch "$deb_arch" packages ./ > Packages

gzip -kf Packages

cat Distributions > Release

apt-ftparchive release . >> Release

gpg --default-key "$GPG_SIGNING_EMAIL" --clearsign --yes -o InRelease Release
gpg --default-key "$GPG_SIGNING_EMAIL" --armor --detach-sign --sign --yes -o Release.gpg Release

mkdir artifacts
cp InRelease Release Release.gpg Packages Packages.gz ./artifacts/
gh release upload debian ./artifacts/* --clobber -R nibrobb/luxafor-ui