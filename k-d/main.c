#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "utils.h"
#include "k-d.h"
#define MAX 50

int dist(int *point1, int *point2, int k) {
    int ans = 0;
    for (int i = 0; i < k; ++i)
        ans += (point1[i] - point2[i]) * (point1[i] - point2[i]);

    return ans;
}

int main() {
    kd_tree_t *kd = (kd_tree_t *)calloc(1, sizeof(kd_tree_t));
    DIE(!kd, "Calloc");

    char buff[MAX]; /* load from file the data set */
    scanf("%s", buff);
    scanf("%s", buff);
    load(kd, buff);

    kd->dist_fun = dist;

    while(1) {
        scanf("%s", buff);
        if (strstr(buff, "NN")) { /* search the nearest neighbour */
            int *point = (int *)malloc(kd->k * sizeof(int));
            DIE(!point, "malloc");
            for (int i = 0; i < kd->k; ++i)
                scanf("%d", point + i);

            nearest_neighbour(kd, point);
            free(point);

        } else if (strstr(buff, "RS")) { /* range search */
            int *left, *right;
            left = (int *)malloc(kd->k * sizeof(int));
            DIE(!left, "malloc");

            right = (int *)malloc(kd->k * sizeof(int));
            DIE(!right, "malloc");

            for (int i = 0; i < kd->k; ++i)
                scanf("%d %d", left + i, right + i);

            range_search(kd, left, right);
            free(left);
            free(right);

        } else if (strstr(buff, "EXIT")) { /* deallocates all and exits */
            free_all(kd);
            free(kd);
            break;
        }
    }
    return 0;
}
