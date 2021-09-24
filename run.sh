cargo build --release
#rm ./client*.log
for i in {0..6}; do
#round cts
#trickle -u 1024 -d 1024 ./target/release/client 2 1 &
#./target/release/client 2 1 2>&1 1>/dev/null  &
./target/release/client 2 1 2>&1 1>client$i.log &
done
./target/release/client 2 1
wait
