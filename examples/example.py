#!/bin/env python
N = input()
arr: list[int] = []
for i in range(0, int(N)):
    arr.append(int(input()))

for i in range(0, int(N)):
    print(arr[i] + 1)
