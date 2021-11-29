#! /bin/bash
app=$1
BASE_DIR="/home/ubuntu/quail"
WORKING_DIR="/home/ubuntu/quail/atom"
CARGO="/home/ubuntu/.cargo/bin/cargo"

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
cd ${WORKING_DIR} && ${CARGO} build --release
ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${WORKING_DIR} && ${CARGO} build --release"

# update the config file and running scripts
# update the config
scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/config.yaml ubuntu@${w}:${BASE_DIR}
# update the script
scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/run_committee.sh ubuntu@${w}:${BASE_DIR}

# start running the aggregator
cd ${BASE_DIR}
./atom/target/release/aggregator_$app 2>&1 >/dev/null &
# start running the light_client
# ssh -i ${BASE_DIR}/data/aws01.pem ${light_client} "cd ${WORKING_DIR} && ./target/release/light_client 130 2>&1 > light_client.log" &
#pssh  -i  -H "${waddr_list}"  -x "-oStrictHostKeyChecking=no  -i ${BASE_DIR}/data/aws01.pem" "cd ${BASE_DIR} && ./test.sh $app"
# update the config file and running scripts
ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${BASE_DIR} && ./run_committee.sh $app"

wait
#sudo pkill -P $$
