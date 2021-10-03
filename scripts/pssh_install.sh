#!/bin/bash
QUAIL=/home/ubuntu/quail

while read line
do
	echo $line
	scp -i ${QUAIL}/data/aws01.pem -r ${QUAIL} ubuntu@$line:${QUAIL}
	ssh -i ${QUAIL}/data/aws01.pem ubuntu@$line "sudo su && ${QUAIL}/scripts/iinstall_hhoneycrisp.sh"
	#scp -i ${QUAIL}/data/aws01.pem ${QUAIL}/data/id_rsa.pub ubuntu@$line:/home/ubuntu/.ssh/
	#ssh -i ${QUAIL}/data/aws01.pem ubuntu@$line "chmod 400 /home/ubuntu/.ssh/id_rsa"
	#ssh -i ${QUAIL}/data/aws01.pem ubuntu@$line -t "git clone git@github.com:lonhuen/quail.git"
	#ssh -i ${QUAIL}/data/aws01.pem -t  ubuntu@$line "cd /home/ubuntu/quail; ./scripts/install.sh"
done < ${QUAIL}/scripts/worker.txt
