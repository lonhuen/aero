addr=("172.31.44.56" "172.31.35.126" "172.31.40.85")

timestamp=$(date +"%H_%M_%S")
LOG_DIR=logs/$timestamp
mkdir -p ${LOG_DIR}

cp config.yaml ${LOG_DIR}

for a in "${addr[@]}"; do
	mkdir -p ${LOG_DIR}/$a
	scp -i ./data/aws01.pem ubuntu@$a:'/home/ubuntu/quail/client*.log' /home/ubuntu/quail/${LOG_DIR}/$a
	scp -i ./data/aws01.pem ubuntu@$a:'/home/ubuntu/quail/total.log' /home/ubuntu/quail/${LOG_DIR}/$a
	scp -i ./data/aws01.pem ubuntu@$a:'/home/ubuntu/quail/committee*.log' /home/ubuntu/quail/${LOG_DIR}/$a
done

