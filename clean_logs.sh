addr=("172.31.44.56" "172.31.35.126" "172.31.40.85")

for a in "${addr[@]}"; do
	ssh -i ./data/aws01.pem ubuntu@$a 'rm /home/ubuntu/quail/client*.log' 
	ssh -i ./data/aws01.pem ubuntu@$a 'rm /home/ubuntu/quail/total.log'
	ssh -i ./data/aws01.pem ubuntu@$a 'rm /home/ubuntu/quail/committee*.log'
done

