if [ "$(uname)" == "Darwin" ]; then
  echo "Building for MacOS"
  cargo build && cp target/debug/libvimscape_backend.dylib ../lua/vimscape_backend.so
else
  echo "Building for Linux"
  cargo build && cp target/debug/libvimscape_backend.so ../lua/vimscape_backend.so
fi
