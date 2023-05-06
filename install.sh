echo "Installing Rust"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

echo "Downloading checker"
cargo install checker-tema-3-sd

ln -s ~/.cargo/bin/checker-tema-3-sd .

echo "Install finished. Good luck!"