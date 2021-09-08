for i in {0..3}; do
#round cts
./target/release/client 2 1 &
done
wait
