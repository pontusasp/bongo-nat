FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    patchelf \
    build-essential \
    pkg-config \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV APPIMAGE_EXTRACT_AND_RUN=1

# Pre-populate /root/.cache/tauri with linuxdeploy tools so container runs don't re-download them
RUN mkdir -p /root/.cache/tauri \
    && wget -q -O /root/.cache/tauri/AppRun-x86_64 \
      "https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-x86_64" \
    && wget -q -O /root/.cache/tauri/linuxdeploy-x86_64.AppImage \
      "https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy-07333c6/linuxdeploy-x86_64.AppImage" \
    && wget -q -O /root/.cache/tauri/linuxdeploy-plugin-appimage.AppImage \
      "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage" \
    && chmod +x /root/.cache/tauri/AppRun-x86_64 \
      /root/.cache/tauri/linuxdeploy-x86_64.AppImage \
      /root/.cache/tauri/linuxdeploy-plugin-appimage.AppImage

WORKDIR /app

# Default command builds the app and copies the bundled artifacts (.AppImage, .deb, .rpm) into /app/dist/
CMD ["bash", "-c", "npm install && npm run tauri build && mkdir -p /app/dist && cp -f /app/src-tauri/target/release/bundle/appimage/*.AppImage /app/src-tauri/target/release/bundle/deb/*.deb /app/src-tauri/target/release/bundle/rpm/*.rpm /app/dist/ && ls -lh /app/dist/"]

# Usage (works with podman or docker):
#
# $ podman build -t tauri-builder .
#
# $ podman run --rm \
#   -v $(pwd):/app \
#   -v tauri-cargo-cache:/root/.cargo/registry \
#   -v tauri-target-cache:/app/src-tauri/target \
#   tauri-builder
