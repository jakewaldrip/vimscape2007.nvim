# TODO: Create a proper production build process (e.g., build-release.sh or Makefile)
# This script is for development and testing purposes only - it builds in debug mode
# and manually copies the shared library. A real build should:
# - Build with --release flag for optimizations
# - Handle cross-platform builds properly
# - Integrate with plugin managers (lazy.nvim, packer, etc.)
# - Consider CI/CD for pre-built binaries

if [ "$(uname)" == "Darwin" ]; then
  echo "Building for MacOS"
  cargo build && cp target/debug/libvimscape_backend.dylib ../lua/vimscape_backend.so
else
  echo "Building for Linux"
  cargo build && cp target/debug/libvimscape_backend.so ../lua/vimscape_backend.so
fi
