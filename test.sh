APP=$1
echo $APP
cargo build --release
./target/release/aggregator_$APP 2>&1 > aggregator.log &
for i in {0..8}; do 
(./target/release/committee_$APP $i 2>&1 > co$i.log ) &
done
./target/release/committee_$APP 9 | tee co9.log
wait
