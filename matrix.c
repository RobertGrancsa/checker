#include <stdlib.h>
#include <time.h>
#include "matrix.h"
#include "utils-kd.h"

int **alloc(int lines, int columns) {
	int **a = (int **)malloc(lines * sizeof(int *));
	for (int i = 0; i < lines; ++i)
		a[i] = (int *)malloc(columns * sizeof(int));

	return a;
}

void dealloc(int **a, int lines) {
	for (int i = 0; i < lines; ++i)
		free(a[i]);

	free(a);
}

void shuffle(int **a, int lines) {
	srand(time(NULL));
	for (int repeat_no = 0; repeat_no < 2; ++repeat_no) {
		for (int i = lines - 1; i > 0; --i) {
			int j = rand() % (i + 1);

			swap(&a[i], &a[j]);
		}
	}
}
