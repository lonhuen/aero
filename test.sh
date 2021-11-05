
cargo build --release
./target/release/aggregator &
for i in {0..8}; do 
#  echo $i
  ./target/release/committee_offline $i > /dev/null 2> /dev/null &
done
./target/release/committee_offline 9
