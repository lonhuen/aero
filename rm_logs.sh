#addr=("172.31.47.222" "172.31.47.163" "172.31.37.209" "172.31.47.34")
addr=("aws01" "aws02" "aws03" "aws04") 

for a in "${addr[@]}"; do
	ssh $a "rm -r /home/ubuntu/quail/client*.log"
done


#scp -i ./data/aws01.pem config.ini ubuntu@172.31.47.163:/home/ubuntu/quail
#scp -i ./data/aws01.pem config.ini ubuntu@172.31.37.209:/home/ubuntu/quail
#scp -i ./data/aws01.pem config.ini ubuntu@172.31.47.34:/home/ubuntu/quail
