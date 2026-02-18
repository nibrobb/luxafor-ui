#!/usr/bin/env bash

set -x

# TODO: make this more dynamic
luxafor_ui_deb='Luxafor-ui_0.1.0_amd64.deb'

# Get the OS code-name ("noble" on Ubuntu 24.04 LTS)
. /etc/os-release
codename="$VERSION_CODENAME"
# TODO: Make this also more dynamic
deb_arch='amd64'

mkdir -p "dists/$codename/main/binary-amd64"
mkdir -p pool/main/luxafor-ui/

cp Distributions "dists/$codename" || exit 1
cp "../target/release/bundle/deb/$luxafor_ui_deb" "pool/main/$luxafor_ui_deb" || exit 1

apt-ftparchive --arch $deb_arch packages pool/ > "dists/$codename/main/binary-$deb_arch/Packages"
gzip -kf "dists/$codename/main/binary-$deb_arch/Packages"

cd "dists/$codename"
cat Distributions > Release
apt-ftparchive release . >> Release
gpg --clearsign --yes -o InRelease Release
gpg -abs --yes -o Release.gpg Release
