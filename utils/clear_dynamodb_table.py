# Usage:
# 1. Make sure the terminal from which you invoke is correclty configured
#    a. AWS credentials for the AWS account are present in the environment
#    b. The associated permissions allow dynamodb:Scan and dynamodb:BatchWriteItem
#    c. AWS_DEFAULT_REGION is set on the table's region
# 2. Just invoke:
# clear_dynamodb_table.py <table_name>


import boto3
import sys

def clear_table(table_name):
    print(f"Clearing all the content of DynamoDB table: {table_name}")
    # Create table resource
    table = boto3.resource("dynamodb").Table(table_name)

    # Items counter
    count = 0

    #Batch delete
    with table.batch_writer() as batch:
        # Scan the table and batch delete
        scan_params = {
            'ProjectionExpression': 'tatoo',
            'ReturnConsumedCapacity': 'NONE',
        }
        lek = None
        while True:
            if lek is not None:
                scan_params['ExclusiveStartKey'] = lek
            resp = table.scan(**scan_params)
            keys = resp['Items']
            count += len(keys)
            print(f"Clearing {len(keys)} keys")
            for key in keys:
                batch.delete_item(Key=key)
            lek = resp.get('LastEvaluatedKey')
            if lek is None:
                break


    print(f"Table {table_name} emptied")
    print(f"{count} items cleared")

if __name__ == "__main__":
   clear_table(sys.argv[1])
