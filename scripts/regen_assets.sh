#!/bin/bash

# Smol convenience script to regen the icon
#
# Requires imagemagick

magick \
    assets/icon/icon256.png \
    -define icon:auto-resize=16,32,64,128,256 \
    -gravity center \
    -flatten \
    -colors 256 \
    -background transparent \
    assets/icon/icon.ico