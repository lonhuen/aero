APP=$1
echo $APP
cargo build --release
./target/release/aggregator_$APP > aggregator.log &
for i in {0..8}; do 
(./target/release/committee_$APP $i > co$i.log ) &
done
./target/release/committee_$APP 9 | tee co9.log
wait
