import math

# This class represents a Circle Calculator that can calculate the area and circumference of a circle given its radius
class CircleCalculator:

    def __init__(self, radius):
        self.radius = radius
        self.pi = 3.1415

    def calc_area(self):
        r = self.radius
        pi = self.pi
        a = pi * r * r
        return a

    def calc_circumference(self):
        r = self.radius
        pi = 3.1415
        c = 2 * pi * r
        return c
