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

waddr=("172.31.44.56" "172.31.35.126")
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
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/config.yaml ${w}:${BASE_DIR}/config.yaml
	# update the script
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/scripts/exp.sh ${w}:/home/ubuntu/quail/scripts/
done

# start running the server
./$app/target/release/server &
# ./$app/target/release/light_client 1000 &

# update the config file and running scripts
for w in ${waddr[@]}; do
	ssh -i ${BASE_DIR}/data/aws01.pem "cd ${BASE_DIR} && ./scripts.sh $app 15" &
done

wait
#sudo pkill -P $$