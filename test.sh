#APP=$1
#echo $APP
#cargo build --release
#
#start=$(cat /proc/net/dev | grep "lo")
#in_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $2 }')
#out_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $10 }')
#
#./target/release/aggregator_$APP 2>&1 > aggregator.log &
#for i in {0..26}; do 
#(./target/release/committee_$APP $i 2>&1 > co$i.log ) &
#done
#time(./target/release/committee_$APP 27) | tee co27.log
#wait
#
#end=$(cat /proc/net/dev | grep "lo")
#in_bytes_end=$(echo $end| awk -v OFS=, '/lo:/ { print $2 }')
#out_bytes_end=$(echo $end | awk -v OFS=, '/lo:/ { print $10 }')
#echo "recv bytes " $((in_bytes_end - in_bytes))
#echo "sent bytes " $((out_bytes_end - out_bytes))

#! /bin/bash
app=$1
BASE_DIR="/home/ubuntu/quail"
WORKING_DIR="/home/ubuntu/quail/atom"

if [ ! -d "${BASE_DIR}" ]; then
	echo "${BASE_DIR} doesn't exist. Clone the repo and install depences first"
	exit
fi

if [[ "$app" != "offline" ]] && [[ "$app" != "online" ]]; then
	echo "./run_committee.sh online/offline"
	exit
fi

w="172.31.40.85"

# build first
cd ${WORKING_DIR} && cargo build --release
ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${WORKING_DIR} && cargo build --release"

# update the config file and running scripts
# update the config
scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/config.yaml ubuntu@${w}:${BASE_DIR}
# update the script
scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/run_committee.sh ubuntu@${w}:${BASE_DIR}

# start running the aggregator
./target/release/aggregator_$app &
# start running the light_client
# ssh -i ${BASE_DIR}/data/aws01.pem ${light_client} "cd ${WORKING_DIR} && ./target/release/light_client 130 2>&1 > light_client.log" &
#pssh  -i  -H "${waddr_list}"  -x "-oStrictHostKeyChecking=no  -i ${BASE_DIR}/data/aws01.pem" "cd ${BASE_DIR} && ./test.sh $app"
# update the config file and running scripts
ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${BASE_DIR} && ./run_committee.sh $app"

wait
#sudo pkill -P $$