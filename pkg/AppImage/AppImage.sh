#!/bin/sh

# You might need to restart your pc if sharun doesn't create `AppDir` in this directory (It should create dirs on its own)

set -eu

ARCH="$(uname -m)"
SHARUN="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/main/useful-tools/quick-sharun.sh"

export ADD_HOOKS="self-updater.bg.hook"
#export UPINFO="gh-releases-zsync|${GITHUB_REPOSITORY%/*}|${GITHUB_REPOSITORY#*/}|latest|*$ARCH.AppImage.zsync"
export OUTNAME=metapac-anylinux-"$ARCH".AppImage
export DESKTOP=../../metapac.desktop
export ICON=DUMMY
export OUTPATH=./dist
export DEPLOY_OPENGL=0
export DEPLOY_VULKAN=0
export DEPLOY_DOTNET=0

#Remove leftovers
rm -rf AppDir dist appinfo

# ADD LIBRARIES
wget --retry-connrefused --tries=30 "$SHARUN" -O ./quick-sharun
chmod +x ./quick-sharun

# Point to binaries
./quick-sharun /usr/bin/metapac

# Make AppImage
./quick-sharun --make-appimage

echo "All Done!"
