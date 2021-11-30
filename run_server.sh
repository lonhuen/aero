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
#cd ${WORKING_DIR} && cargo build --release
#for w in ${waddr[@]}; do
#	ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${WORKING_DIR} && cargo build --release"
#done

# update the config file and running scripts
for w in ${waddr[@]}; do
	# update the config
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/config.yaml ubuntu@${w}:${BASE_DIR}/
	# update the script
	scp -i ${BASE_DIR}/data/aws01.pem ${BASE_DIR}/scripts/exp.sh ubuntu@${w}:${BASE_DIR}/scripts
done

cd ${BASE_DIR}
# start running the server
./$app/target/release/server &
# ./$app/target/release/light_client 1000 &

# update the config file and running scripts
for w in ${waddr[@]}; do
	ssh -i ${BASE_DIR}/data/aws01.pem ubuntu@${w} "cd ${BASE_DIR} && ./scripts/exp.sh $app 15"  2>/dev/null >/dev/null &
done

wait
#sudo pkill -P $$
