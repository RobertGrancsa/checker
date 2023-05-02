#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "matrix.h"
#include "utils.h"
#define INF 1e9
#define MAX 50

int dist(int *point1, int *point2, int k) {
    int ans = 0;
    for (int i = 0; i < k; ++i)
        ans += (point1[i] - point2[i]) * (point1[i] - point2[i]);

    return ans;
}

void print(int *point, int k) {
    for (int i = 0; i < k; ++i)
        printf("%d ", point[i]);
    printf("\n");
}

void nearest_neighbour(int **a, int n, int k, int *point) {
    int ans = INF;
    for (int i = 0; i < n; ++i)
        ans = min(ans, dist(a[i], point, k));

    for (int i = 0; i < n; ++i)
        if (dist(a[i], point, k) == ans)
            print(a[i], k);
}

int between(int *point, int *left, int *right, int k) {
    for (int i = 0; i < k; ++i)
        if (point[i] < left[i] || point[i] > right[i])
            return 0;

    return 1;
}

void range_search(int **a, int n, int k, int *left, int *right) {
    for (int i = 0; i < n; ++i)
        if (between(a[i], left, right, k))
            print(a[i], k);
}

void load(int ***a, int *n, int *k, char *file_name) {
    FILE *fin = fopen(file_name, "rt");
    fscanf(fin, "%d %d", n, k);

    *a = alloc(*n, *k);
    for (int i = 0; i < *n; ++i) {
        for (int j = 0; j < *k; ++j) {
            fscanf(fin, "%d", &((*a)[i][j]));
        }
    }

    fclose(fin);
}

int main() {

    char buff[MAX]; /* load from file the data set */
    scanf("%s", buff);
    scanf("%s", buff);
    int **a = NULL, n, k;
    load(&a, &n, &k, buff);

    while(1) {
        scanf("%s", buff);
        if (strstr(buff, "NN")) { /* search the nearest neighbour */
            int *point = (int *)malloc(k * sizeof(int));
            DIE(!point, "malloc");
            for (int i = 0; i < k; ++i)
                scanf("%d", point + i);

            nearest_neighbour(a, n, k, point);
            free(point);

        } else if (strstr(buff, "RS")) { /* range search */
            int *left, *right;
            left = (int *)malloc(k * sizeof(int));
            DIE(!left, "malloc");

            right = (int *)malloc(k * sizeof(int));
            DIE(!right, "malloc");

            for (int i = 0; i < k; ++i)
                scanf("%d %d", left + i, right + i);

            range_search(a, n, k, left, right);
            free(left);
            free(right);

        } else if (strstr(buff, "EXIT")) { /* deallocates all and exits */
            dealloc(a, n);
            break;
        }
    }
    return 0;
}
