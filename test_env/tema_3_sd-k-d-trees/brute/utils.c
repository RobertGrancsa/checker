#include <stdlib.h>
#include "utils.h"

void swap(int **a, int **b) {
    int *aux = *a;
    *a = *b;
    *b = aux;
}

int min(int a, int b) {
    return a < b ? a : b;
}

int cmp(int *p1, int *p2, int k) {
    for (int i = 0; i < k; ++i)
        if (p1[i] != p2[i])
            return p1[i] - p2[i];
        
    return 0;
}

void mergesort(int **a, int k, int left, int right) {
    if (left + 1 >= right)
        return;

    int mid = (left + right) / 2;
    mergesort(a, k, left, mid);
    mergesort(a, k, mid, right);

    int **tmp = malloc((right - left) * sizeof(int *));
    int ij = left, j = left, i = mid;
    while (j < mid && i < right) {
        if (cmp(a[i], a[j], k) < 0)
            tmp[ij++ - left] = a[i++]; 
        else
            tmp[ij++ - left] = a[j++];
    }

    while (j < mid)
        tmp[ij++ - left] = a[j++];

    while (i < right)
        tmp[ij++ - left] = a[i++];

    for (int ind = left; ind < right; ++ind)
        a[ind] = tmp[ind - left];

    free(tmp);
}

