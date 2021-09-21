cargo build --release
#rm ./client*.log
for i in {0..2}; do
#round cts
#trickle -u 1024 -d 1024 ./target/release/client 2 1 &
./target/release/client 2 1 2>&1 1>/dev/null  &
done
./target/release/client 2 1
wait
