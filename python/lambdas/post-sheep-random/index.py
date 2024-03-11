import json
import random
import threading

from utils import logger
from apigatewayutils import basic_error, basic_response, HTTPError, extract_parameters
from dynamodbsheepshed import DynamoDBSheepShed, Sheep, Weight, SheepDuplicationError

def generate_random_weight():
    wmin = Weight.MIN.as_ug()
    wmax = Weight.MAX.as_ug()
    return Weight.from_ug(random.randint(wmin, wmax))

def insert_sheep(event):
    parameters = extract_parameters(event)
    tattoo = int(parameters['Tattoo'])

    logger.info(f"tattoo={tattoo}")
    
    # Here I do not implement a separate generating thread like
    # in the Rust version because I feel it is too penalizing for
    # Python, as thread are POSIX thread, quite costly to launch
    # whereas Rust use green-threads and pre-launched OS threads
    logger.info("generating sheep")
    sheep = Sheep(tattoo, generate_random_weight())

    logger.info("create a shed instance")
    dynamodb_sheep_shed = DynamoDBSheepShed()

    logger.info("inserting sheep")
    try:
        dynamodb_sheep_shed.add_sheep(sheep)
    except SheepDuplicationError as sde:
        raise HTTPError(400, str(sde))
    
    logger.info("success")

    return basic_response(201, {
        'tattoo': sheep.tattoo,
        'weight': sheep.weight.as_ug()
    })

def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    try:
        return insert_sheep(event)
    except HTTPError as e:
        logger.exception(e.message)
        return basic_error(e.code, e.message)
    except:
        logger.exception('Server error')
        return basic_error(500, 'Server error')

    logger.error('Should never get in this part of the code...')
    return basic_error(500, 'Server error')
