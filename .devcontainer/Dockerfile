FROM mcr.microsoft.com/vscode/devcontainers/base:ubuntu

SHELL ["/bin/bash", "-euo", "pipefail", "-c"]

RUN apt-get update; \
    # apt-get upgrade -y; \
    apt-get install -y \
        qemu-kvm \
        libvirt-daemon \
        # kmod \
    ; \
	rm -rf /var/lib/apt/lists/*;
