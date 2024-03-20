#!/bin/bash

usage () {
    echo "Usage: $0 [<OPTIONS>] <API_URL>"
    echo ""
    echo "Repeatedly call DELETE <API_URL>/wolf"
    echo ""
    echo -e "-p|--parallel <task_count>\tThe number of concurrent task to use. (Default: 100)"
    echo -e "-c|--call-count <count>\tThe number of call to make. (Default: 1000)"
    echo ""
    echo "OPTIONS:"
    echo -e "-h|--help\t\t\tShow this help"
}

PARALLEL_TASKS=100
COUNT=1000

POSITIONAL=()
while [[ $# -gt 0 ]]
do
    key="$1"
    case $key in
        -h|--help)
            usage
            exit 0
        ;;
        -p|--parallel)
            PARALLEL_TASKS="$2"
            shift # past argument
            shift # past value
        ;;
        -c|--call-count)
            COUNT="$2"
            shift # past argument
            shift # past value
        ;;
        *)    # unknown option
            POSITIONAL+=("$1") # save it in an array for later
            shift # past argument
        ;;
    esac
done

if [ ${#POSITIONAL[@]} -ne 1 ] ; then
    echo "Exactly one <API_URL> is expected as argument"
    usage
    exit 1
fi
API_URL="${POSITIONAL[0]}"
if [ ${API_URL:${#API_URL}-1} == / ]; then
    API_URL=${API_URL::-1}
fi

timestamp_nano() {
  date +%s%N
}

start_ts=$(timestamp_nano)
seq 1 $COUNT | xargs -Iunused -P$PARALLEL_TASKS curl -s --retry 5 --retry-connrefused -XDELETE "$API_URL/wolf" > /dev/null
end_ts=$(timestamp_nano)
echo Calls took $(( ( $end_ts - $start_ts ) / 1000000 ))ms
