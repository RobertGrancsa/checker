#include "utils.h"

void swap(int **a, int **b) {
    int *aux = *a;
    *a = *b;
    *b = aux;
}

int min(int a, int b) {
    return a < b ? a : b;
}
