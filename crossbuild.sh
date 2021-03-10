docker run --rm \
    --volume "${PWD}":/root/src \
    --workdir /root/src \
    joseluisq/rust-linux-darwin-builder:1.49.0 \
    sh -c "cargo build --release"