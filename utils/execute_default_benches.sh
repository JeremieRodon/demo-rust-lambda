#!/bin/bash

usage () {
    echo "Usage: $0 --rust-api <RUST_API_URL> --python-api <PYTHON_API_URL>"
    echo ""
    echo "Execute the 4 loadtest bench in the order: cat, sheeps, dog, wolf for the two APIs, one after the other"
    echo ""
    echo -e "--rust-api <RUST_API_URL>\tThe API URL of the Rust API"
    echo -e "--python-api <PYTHON_API_URL>\tThe API URL of the Python API"
    echo ""
    echo "OPTIONS:"
    echo -e "-h|--help\t\t\tShow this help"
}

PYTHON_API_URL=""
RUST_API_URL=""

POSITIONAL=()
while [[ $# -gt 0 ]]
do
    key="$1"
    case $key in
        -h|--help)
            usage
            exit 0
        ;;
        --rust-api)
            RUST_API_URL="$2"
            shift # past argument
            shift # past value
        ;;
        --python-api)
            PYTHON_API_URL="$2"
            shift # past argument
            shift # past value
        ;;
        *)    # unknown option
            POSITIONAL+=("$1") # save it in an array for later
            shift # past argument
        ;;
    esac
done

if [ ${#POSITIONAL[@]} -ne 0 ] ; then
    echo "Unknown arguments"
    usage
    exit 1
fi

if [ -z $RUST_API_URL ] ; then
    echo "--rust-api <RUST_API_URL> is mandatory"
    usage
    exit 1
fi

if [ -z $PYTHON_API_URL ] ; then
    echo "--python-api <PYTHON_API_URL> is mandatory"
    usage
    exit 1
fi

echo ./invoke_cat.sh $PYTHON_API_URL
./invoke_cat.sh $PYTHON_API_URL

echo ./invoke_cat.sh $RUST_API_URL
./invoke_cat.sh $RUST_API_URL

echo ./insert_sheeps.sh $PYTHON_API_URL
./insert_sheeps.sh $PYTHON_API_URL

echo ./insert_sheeps.sh $RUST_API_URL
./insert_sheeps.sh $RUST_API_URL

echo ./invoke_dog.sh $PYTHON_API_URL
./invoke_dog.sh $PYTHON_API_URL

echo ./invoke_dog.sh $RUST_API_URL
./invoke_dog.sh $RUST_API_URL

echo ./invoke_wolf.sh $PYTHON_API_URL
./invoke_wolf.sh $PYTHON_API_URL

echo ./invoke_wolf.sh $RUST_API_URL
./invoke_wolf.sh $RUST_API_URL

echo Done.
