BASE_PORT=39999
INCREMENT=1

port=$BASE_PORT
isfree=$(netstat -taln | grep $port)

while [[ -n "$isfree" ]]; do
	    port=$[port+INCREMENT]
	        isfree=$(netstat -taln | grep $port)
	done

	echo "Usable Port: $port"
