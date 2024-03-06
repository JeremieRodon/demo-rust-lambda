import json

from utils import logger
from apigatewayutils import basic_error, basic_response, HTTPError
from dynamodbsheepshed import DynamoDBSheepShed

def bark_answer():
    logger.info("create a shed instance")
    dynamodb_sheep_shed = DynamoDBSheepShed()

    logger.info("counting sheeps...")
    count = dynamodb_sheep_shed.sheep_count()

    logger.info(f"success - count={count}")
    return basic_response(200, {'count': count})

def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    try:
        return bark_answer()
    except HTTPError as e:
        logger.exception(e.message)
        return basic_error(e.code, e.message)
    except:
        logger.exception('Server error')
        return basic_error(500, 'Server error')

    logger.error('Should never get in this part of the code...')
    return basic_error(500, 'Server error')
