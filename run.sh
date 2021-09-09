for i in {0..3}; do
#round cts
trickle -u 1024 -d 1024 ./target/release/client 2 1 &
done
wait
