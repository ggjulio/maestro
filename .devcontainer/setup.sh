#!/bin/bash

set -euo pipefail

main(){
    # TODO: setup everything for using kemu.
    echo "TODO .devcontainer/setup.sh"
    # dd if=/dev/zero of=qemu_disk count=1G status=progress
    # dd if=/dev/zero of=qemu_disk bs=1 count=0 seek=1G status=progress
    # cargo build ?
}

main "$@"