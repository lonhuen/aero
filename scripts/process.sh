#grep  "data downloaded" ./$1/client* --no-filename | awk -F'[: ]' '{print $4}'
#echo
#grep  "data uploaded" ./$1/client*.log --no-filename | awk -F'[: ]' '{print $4}'
#echo
#grep  "verified" ./$1/client*.log --no-filename | awk -F'[: ]' '{print $3}'
#echo
#grep  "Total" ./$1/client*.log --no-filename | awk -F'[: ]' '{print $4}'

grep "recv" ./$1/total.log --no-filename | awk -F'[ ]' '{print $4}'
echo
grep "sent" ./$1/total.log --no-filename | awk -F'[ ]' '{print $4}'
