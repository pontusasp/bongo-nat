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

# Install Tauri CLI
RUN cargo install tauri-cli

# Download and extract appimagetool as a native binary (no FUSE needed)
RUN wget -O /tmp/appimagetool.AppImage \
    "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
    && chmod +x /tmp/appimagetool.AppImage \
    && APPIMAGE_EXTRACT_AND_RUN=1 /tmp/appimagetool.AppImage --appimage-extract \
    && mv squashfs-root /opt/appimagetool \
    && rm /tmp/appimagetool.AppImage

WORKDIR /app

# $ docker build -t tauri-builder .
#
# $ docker run --rm \
#   -v $(pwd):/app \
#   -v tauri-cargo-cache:/root/.cargo/registry \
#   -v tauri-target-cache:/app/src-tauri/target \
#   -e APPIMAGE_EXTRACT_AND_RUN=1 \
#   tauri-builder \
#   bash -c "npm install && \
#     cargo tauri build; \
#     ARCH=x86_64 /opt/appimagetool/AppRun \
#       '/app/src-tauri/target/release/bundle/appimage/Bongo Nat.AppDir' \
#       '/app/src-tauri/target/release/bundle/appimage/Bongo Nat_X.Y.Z_amd64.AppImage'"
