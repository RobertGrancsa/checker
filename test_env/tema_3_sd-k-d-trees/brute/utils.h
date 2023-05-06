#ifndef UTILS_H_
#define UTILS_H_

#include <errno.h>

#define DIE(assertion, call_description)				\
	do {								\
		if (assertion) {					\
			fprintf(stderr, "(%s, %d): ",			\
					__FILE__, __LINE__);		\
			perror(call_description);			\
			exit(errno);					\
		}							\
	} while (0)

void swap(int **a, int **b);
int min(int a, int b);
int cmp(int *p1, int *p2, int k);
void mergesort(int **a, int k, int left, int right);

#endif
