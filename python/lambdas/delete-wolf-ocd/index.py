import json
import math
import threading

from utils import logger
from apigatewayutils import basic_error, basic_response, HTTPError
from dynamodbsheepshed import DynamoDBSheepShed, Sheep, Weight


def sieve_of_eratosthenes(n):
    tmp = [True for i in range(n+1)]
    tmp[0] = tmp[1] = False
    sqrt_n = int(math.sqrt(n)) + 1
    for i in range(2, sqrt_n):
        if tmp[i]:
            j = i * i
            while j <= n:
                tmp[j] = False
                j += i
    return [p for (p, b) in enumerate(tmp) if b]

def wolf_ocd():
    sieve = [None]
    sieve_max = int(math.sqrt(Weight.MAX.as_ug()))
    def create_sieve(n):
        sieve[0] = sieve_of_eratosthenes(n)
    logger.info(f"spawning primes sieve generation (2 to {sieve_max})...")
    t_sieve = threading.Thread(target=create_sieve, args=(sieve_max,))
    t_sieve.start()

    logger.info(f"retrieving all the sheeps...")
    sheeps = [None]
    def get_sheeps():
        logger.info(f"create a shed instance")
        sheeps[0] = list(DynamoDBSheepShed().sheep_iter())
    t_sheeps = threading.Thread(target=get_sheeps)
    t_sheeps.start()

    t_sieve.join()
    sieve = sieve[0]
    logger.info(f"sieve contains {len(sieve)} primes")

    t_sheeps.join()
    sheeps = sheeps[0]
    logger.info(f"sheep list contains {len(sheeps)} sheep")
    
    def edible_sheep(sheep):
        sheep_weight_ug = sheep.weight.as_ug()
        for prime in sieve:
            if sheep_weight_ug % prime == 0:
                return False
        return True
    edible_sheeps = [sheep for sheep in sheeps if edible_sheep(sheep)]
    if len(edible_sheeps) > 0:
        sheep_to_eat = max(edible_sheeps, key=lambda o:o.weight)
        logger.info(f"wolf will eat {sheep_to_eat}")
        logger.info(f"create a shed instance")
        try:
            DynamoDBSheepShed().kill_sheep(sheep_to_eat.tatoo)
        except SheepNotPresentError as snpe:
            raise HTTPError(404, str(snpe))
        return basic_response(204)
    else:
        logger.info(f"it seems the wolf will continue to starve...")
        return basic_response(404, {'message': "No fitting sheep"})


def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    try:
        return wolf_ocd()
    except HTTPError as e:
        logger.exception(e.message)
        return basic_error(e.code, e.message)
    except:
        logger.exception('Server error')
        return basic_error(500, 'Server error')

    logger.error('Should never get in this part of the code...')
    return basic_error(500, 'Server error')
