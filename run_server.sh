#! /bin/bash
app=$1
BASE_DIR="/home/ubuntu/quail"
WORKING_DIR="/home/ubuntu/quail/$app"

if [ ! -d "${BASE_DIR}" ]; then
	echo "${BASE_DIR} doesn't exist. Clone the repo and install depences first"
	exit
fi

if [[ "$app" != "atom" ]] && [[ "$app" != "baseline" ]]; then
	echo "./run_server atom/baseline"
	exit
fi

waddr=("ubuntu@172.31.40.188" "ubuntu@172.31.40.188")
light_client="ubuntu@172.31.40.188"
waddr_list="${waddr[@]}"
echo "worker list"
echo $waddr_list

# build first
cd ${WORKING_DIR}
cargo build --release
pssh  -i  -H "${waddr_list}"  -x "-oStrictHostKeyChecking=no  -i ${BASE_DIR}/data/aws01.pem" "cd ${WORKING_DIR}; cargo build --release"

# update the config file and running scripts
for w in ${waddr[@]}; do
	# update the config
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/config.yaml ubuntu@${w}:${BASE_DIR}/config.yaml
	# update the script
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/scripts/exp.sh ubuntu@${w}:/home/ubuntu/quail/scripts/
done

# start running the server
./$app/target/release/server &
# start running the light_client
ssh -i ${BASE_DIR}/data/aws01.pem ${light_client} "cd ${WORKING_DIR} && ./target/release/light_client 130 2>&1 > light_client.log" &
pssh  -i  -H "${waddr_list}"  -x "-oStrictHostKeyChecking=no  -i ${BASE_DIR}/data/aws01.pem" "cd ${BASE_DIR} && ./scripts/exp.sh $app"

wait
#sudo pkill -P $$