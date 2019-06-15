declare start="${1:-2019-06-15 06:00:00 +0100}"
declare end="${2:-2019-06-16 01:00:00 +0800}"

curl -X POST http://localhost:8080/ -d '{ "start": "'"$start"'", "end": "'"$end"'" }'

while true
do
    declare json=$(curl -s http://localhost:8080/)
    declare elapsed=$(echo $json | jq -r '.elapsed_s/60/60 | round')
    declare walltime=$(echo $json | jq -r '.walltime')
    declare remaining=$(echo $json | jq -r '.remaining_s/60/60 | round')

    echo "$elapsed | $walltime | $remaining"

    sleep 1
done
