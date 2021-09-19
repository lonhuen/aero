scp -i ./data/aws01.pem ./config.ini ubuntu@172.31.47.222:quail/
cargo run --bin server --release
