#!/bin/bash

tempdir=$(mktemp -d)
cp "$(dirname "$(realpath "$0")")/gamescopectl-fixture.sh" "${tempdir}/gamescopectl"
chmod +x "${tempdir}/gamescopectl"
export PATH="${tempdir}:${PATH}"
export XDG_PICTURES_DIR="${tempdir}"
$@
rm -rf "${tempdir}"
