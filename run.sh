cargo build --release

nr_real=$(grep "^nr_real" ./config.ini | awk -F[=] '{print $2}')
#rm ./client*.log
cargo run --bin light_client --release 2>&1 1>light_client.log &

for i in `seq 0 $((nr_real-2))`; do
##./target/release/client 2 1 2>&1 1>/dev/null  &
  ./target/release/client 2>&1 1>client$i.log &
done
./target/release/client 
wait
