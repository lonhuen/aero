scp -i ./data/aws01.pem config.ini ubuntu@172.31.47.222:/home/ubuntu/quail
scp -i ./data/aws01.pem config.ini ubuntu@172.31.47.163:/home/ubuntu/quail
scp -i ./data/aws01.pem config.ini ubuntu@172.31.37.209:/home/ubuntu/quail
scp -i ./data/aws01.pem config.ini ubuntu@172.31.47.34:/home/ubuntu/quail

scp -i ./data/aws01.pem scripts/exp.sh ubuntu@172.31.47.222:/home/ubuntu/quail/scripts/
scp -i ./data/aws01.pem scripts/exp.sh ubuntu@172.31.47.163:/home/ubuntu/quail/scripts/
scp -i ./data/aws01.pem scripts/exp.sh ubuntu@172.31.37.209:/home/ubuntu/quail/scripts/
scp -i ./data/aws01.pem scripts/exp.sh ubuntu@172.31.47.34:/home/ubuntu/quail/scripts/

#ssh -i ./data/aws01.pem ubuntu@172.31.47.222 "bash ~/quail/scripts/network.sh > /dev/null 2>/dev/null" &
#ssh -i ./data/aws01.pem ubuntu@172.31.47.163 "bash ~/quail/scripts/network.sh > /dev/null 2>/dev/null" &
#ssh -i ./data/aws01.pem ubuntu@172.31.37.209 "bash ~/quail/scripts/network.sh > /dev/null 2>/dev/null" &
ssh -i ./data/aws01.pem ubuntu@172.31.47.222 "~/quail/scripts/exp.sh 3 > /dev/null 2>/dev/null" &
ssh -i ./data/aws01.pem ubuntu@172.31.47.163 "~/quail/scripts/exp.sh 3 > /dev/null 2>/dev/null" &
ssh -i ./data/aws01.pem ubuntu@172.31.37.209 "~/quail/scripts/exp.sh 4 > /dev/null 2>/dev/null" &
#ssh -i ./data/aws01.pem ubuntu@172.31.47.34 "source ~/.cargo/env; cd ~/quail && cargo run --bin light_client --release 130 2>&1 > light_client.log"&
#ssh -i ./data/aws01.pem ubuntu@172.31.47.34 "source ~/.cargo/env; cd ~/quail && cargo run --bin light_client --release 130 2>&1 > light_client.log"&
#ssh -i ./data/aws01.pem ubuntu@172.31.47.34 "source ~/.cargo/env; cd ~/quail && cargo run --bin light_client --release 130 2>&1 > light_client.log"&
#ssh -i ./data/aws01.pem ubuntu@172.31.47.34 "source ~/.cargo/env; cd ~/quail && cargo run --bin light_client --release 100 2>&1 > light_client.log"&
#(cargo run --bin light_client --release 490 2>&1 > light_client.log) &
cargo run --bin server --release
wait
sudo pkill -P $$
