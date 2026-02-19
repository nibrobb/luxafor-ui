#!/usr/bin/env bash

set -x

# sudo apt update
# sudo apt install -y wget git gpg apt-utils dpkg-dev
# git checkout v${{ github.event.inputs.version }}
# cd ${{ github.workspace }}
# gh release download debian -D debian -R nibrobb/luxafor-ui
# mv -v ./*.deb ./debian # hmm
# cd debian

# Get the OS code-name ("noble" on Ubuntu 24.04 LTS)
codename="$(lsb_release -sc 2>/dev/null)"
# TODO: Make this also more dynamic
deb_arch='amd64'

mkdir -p "dists/$codename/main/binary-$deb_arch"
mkdir -p pool/main/luxafor-ui/

sed -i "s/%codename%/$codename/g" Distributions
sed -i "s/%arch%/$deb_arch/g" Distributions
cat <<EOF > Distributions
Origin: luxafor-ui
Label: nibrobb luxafor-ui
Suite: $codename
Codename: $codename
Architectures: $deb_arch
Components: main
Description: Graphical desktop application for controlling a Luxafor FLAG­™
EOF

cp Distributions "dists/$codename/" || exit 1
cp "../target/release/bundle/deb/*.deb" pool/main/luxafor-ui/ || exit 1

apt-ftparchive --arch $deb_arch packages pool/ > "dists/$codename/main/binary-$deb_arch/Packages"
gzip -kf "dists/$codename/main/binary-$deb_arch/Packages"

cd "dists/$codename"
cat Distributions > Release
apt-ftparchive release . >> Release
gpg --default-key "${GPG_SIGNING_EMAIL}" --clearsign --yes -o InRelease Release
gpg --default-key "${GPG_SIGNING_EMAIL}" --armor --detach-sign --sign --yes -o Release.gpg Release


# gh release upload debian ./* --clobber -R nibrobb/luxafor-ui