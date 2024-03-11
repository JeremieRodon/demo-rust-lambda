class Weight:
    def __init__(self, weight):
        self.__weight = weight
    
    @classmethod
    def from_kg(cls, weight_kg):
        return cls(int(weight_kg * 1_000_000_000))
    
    @classmethod
    def from_g(cls, weight_g):
        return cls(int(weight_g * 1_000_000))
    
    @classmethod
    def from_mg(cls, weight_mg):
        return cls(int(weight_mg * 1_000))
    
    @classmethod
    def from_ug(cls, weight_ug):
        return cls(int(weight_ug))
    
    def as_kg(self):
        return self.__weight / 1_000_000_000
    
    def as_g(self):
        return self.__weight / 1_000_000
    
    def as_mg(self):
        return self.__weight / 1_000
    
    def as_ug(self):
        return self.__weight
    
    def __str__(self):
        if (self.__weight > 1_000_000_000):
            return "{:.3f}kg".format(self.as_kg())
        elif self.__weight > 1_000_000:
            return "{:.3f}g".format(self.as_g())
        elif  self.__weight > 1_000:
            return "{:.3f}mg".format(self.as_mg())
        else:
            return "{}ug".format(self.__weight)
    
    def __add__(self, rhs):
        return Weight(self.__weight + rhs.__weight)
    
    def __eq__(self, rhs):
        return self.__weight == rhs.__weight
    
    def __le__(self, rhs):
        return self.__weight <= rhs.__weight
    
    def __lt__(self, rhs):
        return self.__weight < rhs.__weight
    
    def __ge__(self, rhs):
        return self.__weight >= rhs.__weight
    
    def __gt__(self, rhs):
        return self.__weight > rhs.__weight

Weight.MIN=Weight(80_000_000_000)
Weight.MAX=Weight(160_000_000_000)

class Sheep:
    def __init__(self, tattoo, weight):
        self.tattoo = tattoo
        self.weight = weight
    
    def __str__(self):
        return f"Sheep({self.tattoo}) weighting {self.weight}"
    
    def __eq__(self, rhs):
        return self.tattoo == rhs.tattoo
