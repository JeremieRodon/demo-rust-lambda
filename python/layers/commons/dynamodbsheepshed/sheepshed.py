import boto3
import os
import concurrent.futures
from utils import logger

from .sheep import Sheep, Weight
from .errors import SheepDuplicationError, GenericError, SheepNotPresentError


class DynamoDBSheepShed:
    def __init__(self):
        self.__table_name = os.environ['BACKEND_TABLE_NAME']
        self.__table = boto3.resource('dynamodb').Table(self.__table_name)
        self.__client = self.__table.meta.client
    
    def _full_table_scan(self, count_only):
        logger.info(f"_full_table_scan(count_only={count_only})")
        # Request the approximate item count that DynamoDB updates sometimes
        approx_table_size = self.__client.describe_table(TableName=self.__table_name)['Table']['ItemCount']
        logger.info(f"approx_table_size={approx_table_size}")

        # The database representation of a Sheep is ~50 bytes
        # Therefore, a scan page of 1MB will contain ~20 000 sheeps
        # We will be very conservative and devide the table into 100k sheeps segments because
        # it would be very counter-productive to have more parallel scans than we would have had pages.
        # Bottom line, with 100k items per segment we expect each segment to be fully scanned in 5 requests.
        parallel_scan_threads = min(int(1 + approx_table_size / 100_000), 1_000_000)
        def scan_segment(seg_i, total_segments):
            items = [] if not count_only else None
            count = 0
            exclusive_start_key = None
            scan_args = {
                'Segment': seg_i,
                'TotalSegments': total_segments,
                'Select': "COUNT" if count_only else "ALL_ATTRIBUTES",
            }
            while True:
                if exclusive_start_key is not None:
                    scan_args['ExclusiveStartKey'] = exclusive_start_key
                try:
                    results = self.__table.scan(**scan_args)
                    exclusive_start_key = results.get('LastEvaluatedKey')
                    count += results['Count']
                    if not count_only:
                        items.extend([
                            Sheep(tattoo=ditem['tattoo'], weight=Weight(ditem['weight']))
                            for ditem in results['Items']
                        ])
                    if exclusive_start_key is None:
                        break
                except Exception as e:
                    raise GenericError(str(e))
            return count, items
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=parallel_scan_threads) as executor:
            tasks = [executor.submit(scan_segment, i, parallel_scan_threads) for i in range(parallel_scan_threads)]
            logger.info(f"Launched {len(tasks)} python-threads")
            items = [] if not count_only else None
            count = 0
            for task in concurrent.futures.as_completed(tasks):
                t_count, t_items = task.result()
                count += t_count
                if not count_only:
                    items.extend(t_items)

        logger.info(f"_full_table_scan => ({count}, {'Some' if not count_only else 'None'})")
        return count, items
    
    def add_sheep(self, sheep):
        """
        Add a new [Sheep] in the [SheepShed]
        # Errors
        It is not allowed to add a duplicated [Sheep], will raise a
        [SheepDuplicationError] if the user tries to add
        a [Sheep] with an already known [Tattoo]
        """
        logger.info(f"add_sheep(sheep={sheep})")
        try:
            self.__table.put_item(
                Item={
                    'tattoo': sheep.tattoo,
                    'weight': sheep.weight.as_ug(),
                },
                ConditionExpression='attribute_not_exists(tattoo)',
                ReturnValues='NONE'
            )
        except self.__client.exceptions.ConditionalCheckFailedException:
            raise SheepDuplicationError(sheep.tattoo)
        except Exception as e:
            raise GenericError(str(e))
        logger.info(f"add_sheep => Ok")

    def sheep_count(self):
        """Return the number of [Sheep] in the [SheepShed]"""
        return self._full_table_scan(True)[0]
    
    def sheep_iter(self):
        """Return an [Iterator] over all the [Sheep]s in the [SheepShed]"""
        return iter(self._full_table_scan(False)[1])

    def kill_sheep(self, tattoo):
        """
        Kill an unlucky Sheep.
        Remove it from the [SheepShed] and return it's body.
        # Errors
        It is not allowed to kill an inexistant [Sheep], will raise
        a [SheepNotPresent] if the user tries to kill a [Sheep] that
        is not in the [SheepShed]
        """
        logger.info(f"kill_sheep(tattoo={tattoo})")
        try:
            result = self.__table.delete_item(
                Key={'tattoo': tattoo},
                ConditionExpression='attribute_exists(tattoo)',
                ReturnValues='ALL_OLD'
            )
            sheep = Sheep(tattoo=result['Attributes']['tattoo'], weight=Weight(result['Attributes']['weight']))
            logger.info(f"kill_sheep => {sheep}")
            return sheep
        except self.__client.exceptions.ConditionalCheckFailedException:
            raise SheepNotPresentError(tattoo)
        except Exception as e:
            raise GenericError(str(e))
