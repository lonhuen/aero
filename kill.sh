pkill -f 'target/release/server$'
#for i in {1..4}; do
#	#ssh aws0$i "sudo pkill tt;sudo pkill target/release/client"
#	ssh aws0$i "sudo pkill target/release/client"
#done
pkill committee_offline
pkill aggregator
