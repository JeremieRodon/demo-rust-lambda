import json

from utils import logger
from apigatewayutils import basic_error, basic_response, HTTPError, extract_parameters

def ackermann_2ary_iter(m, n):
    stack = [m, n]
    while len(stack) > 1:
        n, m = stack.pop(), stack.pop()
        if m == 0:
            stack.append(n + 1)
        elif n == 0:
            stack.append(m - 1)
            stack.append(1)
        else:
            stack.append(m - 1)
            stack.append(m)
            stack.append(n - 1)
    return stack[0]


def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    try:
        params = extract_parameters(event)
        m = int(params['m'])
        n = int(params['n'])
        logger.info(f"Running A({m}, {n})...")
        result = ackermann_2ary_iter(m, n)
        logger.info(f"A({m}, {n}) = {result}")
        return basic_response(200, {'result': result})
    except HTTPError as e:
        logger.exception(e.message)
        return basic_error(e.code, e.message)
    except:
        logger.exception('Server error')
        return basic_error(500, 'Server error')

    logger.error('Should never get in this part of the code...')
    return basic_error(500, 'Server error')
