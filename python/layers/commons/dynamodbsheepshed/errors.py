class SheepNotPresentError(Exception):
    def __init__(self, tattoo):
        super().__init__(f"Sheep is not in the shed: {tattoo}")

class SheepDuplicationError(Exception):
    def __init__(self, tattoo):
        super().__init__(f"Sheep already in the shed: {tattoo}")

class GenericError(Exception):
    def __init__(self, message):
        super().__init__(f"Generic error: {message}")
