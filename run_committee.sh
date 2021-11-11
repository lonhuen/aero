#! /bin/bash
ssh -i ./data/aws01.pem ubuntu@172.31.47.222 "~/quail/target/release/committee_offline 0" &

for i in {1..27}; do
	~/quail/target/release/committee_offline $i
done
