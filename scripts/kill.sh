for i in {1..5}; do
	#ssh aws0$i "sudo pkill tt;sudo pkill target/release/client"
	#ssh aws0$i "sudo pkill target/release/client"
	ssh aws0$i "sudo pkill Setup.x"
	ssh aws0$i "sudo pkill Player.x"
done
