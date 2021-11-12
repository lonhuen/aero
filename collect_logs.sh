#addr=("172.31.47.222" "172.31.47.163" "172.31.37.209" "172.31.47.34")
addr=("aws01" "aws02" "aws03" "aws04") 

timestamp=$(date +"%H_%M_%S")
mkdir -p $timestamp

cp config.yaml $timestamp/

for a in "${addr[@]}"; do
	mkdir -p $timestamp/$a
	scp ubuntu@$a:'/home/ubuntu/quail/client*.log' /home/ubuntu/quail/$timestamp/$a
	scp ubuntu@$a:'/home/ubuntu/quail/total.log' /home/ubuntu/quail/$timestamp/$a
done


#scp -i ./data/aws01.pem config.yaml ubuntu@172.31.47.163:/home/ubuntu/quail
#scp -i ./data/aws01.pem config.yaml ubuntu@172.31.37.209:/home/ubuntu/quail
#scp -i ./data/aws01.pem config.yaml ubuntu@172.31.47.34:/home/ubuntu/quail
