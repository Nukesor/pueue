#!/bin/env python3
#
# A small helper script which downloads all artifacts and creates a sha256 sum for it.
#
# Accepts a release tag as the first parameter, e.g. 'v3.1.0'.
# Otherwise it will print the shaws for the latest release.

import sys
import requests
import hashlib

base_url = "https://github.com/Nukesor/pueue/releases/latest/download/"

if len(sys.argv) > 1:
    release = sys.argv[1]
    print(f"Sha256 sums for artifacts of release {release}")
    base_url = f"https://github.com/Nukesor/pueue/releases/download/{release}/"
else:
    print(f"Sha256 sums for artifacts of latest release")

artifacts = [
    "pueue-linux-x86_64",
    "pueued-linux-x86_64",
    "pueue-macos-x86_64",
    "pueued-macos-x86_64",
    "pueued-windows-x86_64.exe",
    "pueue-windows-x86_64.exe",
    "pueue-darwin-aarch64",
    "pueued-darwin-aarch64",
    "pueue-linux-aarch64",
    "pueued-linux-aarch64",
    "pueue-linux-arm",
    "pueued-linux-arm",
    "pueue-linux-armv7",
    "pueued-linux-armv7",
]

for artifact in artifacts:
    url = base_url + artifact
    response = requests.get(url, stream=True)
    sha256_hash = hashlib.sha256()

    if response.status_code == 200:
        for chunk in response.iter_content(4096):
            sha256_hash.update(chunk)

        sha256_sum = sha256_hash.hexdigest()
        print(f"{artifact}: {sha256_sum}")
    else:
        print(f"Failed to download {artifact}. Status code: {response.status_code}")
