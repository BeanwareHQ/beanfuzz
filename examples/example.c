#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
	size_t buf_size = 10;
	char *buf = malloc(buf_size);
	getline(&buf, &buf_size, stdin);

	int N = atoi(buf);
	int arr[N];

	for (int i = 0; i < N; i++) {
		getline(&buf, &buf_size, stdin);
		arr[i] = atoi(buf);
	}

	for (int i = 0; i < N; i++) {
		printf("%d\n", arr[i] + 1);
	}
}
