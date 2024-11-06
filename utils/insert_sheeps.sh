#!/bin/sh

usage () {
    echo "Usage: $0 [<OPTIONS>] <API_URL> [<sheep_count>]"
    echo ""
    echo "Inserts the given amount of sheeps by repeatedly calling POST <API_URL>/sheep/<tattoo>"
    echo "Tattoos will be all the numbers from 1 to <sheep_count> included, unless the -s option is used"
    echo "<sheep_count> defaults to 1000"
    echo ""
    echo -e "-p|--parallel <task_count>\tThe number of concurrent task to use. (Default: 100)"
    echo -e "-s|--start-at <index>\tThe first sheep tattoo to use. (Default: 1)"
    echo ""
    echo "OPTIONS:"
    echo -e "-h|--help\t\t\tShow this help"
}

PARALLEL_TASKS=100
SHEEP_COUNT=1000
START_AT=1

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
        -s|--start-at)
            START_AT="$2"
            shift # past argument
            shift # past value
        ;;
        *)    # unknown option
            POSITIONAL+=("$1") # save it in an array for later
            shift # past argument
        ;;
    esac
done

if [ ${#POSITIONAL[@]} -eq 0 ] ; then
    echo "At least <API_URL> is expected as argument"
    usage
    exit 1
fi

if [ ${#POSITIONAL[@]} -gt 2 ] ; then
    echo "Too much arguments. Arguments unknown"
    usage
    exit 1
fi

API_URL="${POSITIONAL[0]}"
if [ ${API_URL:${#API_URL}-1} == / ]; then
    API_URL=${API_URL::-1}
fi
if [ ${#POSITIONAL[@]} -eq 2 ] ; then
    SHEEP_COUNT="${POSITIONAL[1]}"
fi

timestamp_nano() {
  date +%s%N
}
SEQ_END=$(( $START_AT + $SHEEP_COUNT - 1 ))
start_ts=$(timestamp_nano)
seq $START_AT $SEQ_END | xargs -Itattoo -P$PARALLEL_TASKS curl -s --retry 5 --retry-connrefused -XPOST "$API_URL/sheep/tattoo" > /dev/null
end_ts=$(timestamp_nano)
echo Insertion took $(( ( $end_ts - $start_ts ) / 1000000 ))ms
