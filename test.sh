cargo build --release
#./target/release/aggregator &
for i in {0..8}; do 
#  echo $i
(./target/release/committee_offline $i | tee co$i.log ) &
done
./target/release/committee_offline 9 | tee co9.log
