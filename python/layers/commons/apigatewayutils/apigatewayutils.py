from utils import logger

import json
import os

class HTTPError(Exception):
    def __init__(self, code, message):
        self.code = code
        self.message = message

def get_user_name(event):
    try:
        return event['requestContext']['authorizer']['claims']['sub']
    except:
        raise HTTPError(401, 'Unauthorized')

def get_body_json(event):
    try:
        return json.loads(event['body'])
    except:
        logger.exception('Could not load the body')
        raise HTTPError(400, 'Invalid JSON body')

def get_query_parameters(event):
    qp = event.get('queryStringParameters')
    if qp is None:
        return {}
    return qp

def get_path_parameters(event):
    pp = event.get('pathParameters')
    if pp is None:
        return {}
    return pp

def extract_parameters(event):
    params = {}
    params.update(get_query_parameters(event))
    params.update(get_path_parameters(event))
    return params

def basic_error(code, message):
    return basic_response(code, {'message':message})

def basic_response(code, obj={}):
    resp = {
        'statusCode': code,
        'headers': {
            'Access-Control-Allow-Origin': os.environ['ALLOW_ORIGIN']
        },
        'body': json.dumps(obj, separators=(',', ':'))
    }
    logger.info(f'Sending resp: {resp}')
    return resp
