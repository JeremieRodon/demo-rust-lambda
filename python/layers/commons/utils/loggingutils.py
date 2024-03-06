import logging

log_level = logging.INFO
# create logger
logger = logging.getLogger('common')
logger.setLevel(log_level)
logger.propagate = False
# create console handler
ch = logging.StreamHandler()
ch.setLevel(log_level)
# create formatter
formatter = logging.Formatter('[%(asctime)s][%(threadName)s]%(levelname)s - %(message)s')
# add formatter to ch
ch.setFormatter(formatter)
# add ch to logger
logger.addHandler(ch)
