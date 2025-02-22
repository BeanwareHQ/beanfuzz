#!/bin/env python
# My bad program that needs to be fuzzed.

from random import randint

N = input()
arr: list[int] = []
for i in range(0, int(N)):
    arr.append(int(input()))

if randint(0, 1) == 1:
    correct = True
else:
    correct = False

for i in range(0, int(N)):
    if correct:
        print(arr[i] + 1)
    else:
        print(arr[i] - 1)
