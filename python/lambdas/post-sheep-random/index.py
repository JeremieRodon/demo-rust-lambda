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
    tatoo = int(parameters['Tatoo'])

    logger.info(f"tatoo={tatoo}")
    
    sheep = [None]
    def create_sheep():
        sheep[0]=Sheep(tatoo, generate_random_weight())
    logger.info(f"spawning sheep generation...")
    t_sheep_gen = threading.Thread(target=create_sheep)
    t_sheep_gen.start()

    logger.info("create a shed instance")
    dynamodb_sheep_shed = DynamoDBSheepShed()

    logger.info("waiting sheep generation...")
    t_sheep_gen.join()
    sheep = sheep[0]

    logger.info("inserting sheep")
    try:
        dynamodb_sheep_shed.add_sheep(sheep)
    except SheepDuplicationError as sde:
        raise HTTPError(400, str(sde))
    
    logger.info("success")

    return basic_response(201, {
        'tatoo': sheep.tatoo,
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
