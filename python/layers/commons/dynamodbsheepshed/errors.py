class SheepNotPresentError(Exception):
    def __init__(self, tatoo):
        super().__init__(f"Sheep is not in the shed: {tatoo}")

class SheepDuplicationError(Exception):
    def __init__(self, tatoo):
        super().__init__(f"Sheep already in the shed: {tatoo}")

class GenericError(Exception):
    def __init__(self, message):
        super().__init__(f"Generic error: {message}")
