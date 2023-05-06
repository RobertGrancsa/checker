#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "k-d.h"
#include "matrix.h"
#include "utils.h"

#define INF 1e9

kd_node_t *get_node(int *data, int k) {
    kd_node_t *new_node = (kd_node_t *)calloc(1, sizeof(kd_node_t));
    new_node->data = (int *)malloc(k * sizeof(int));
    memcpy(new_node->data, data, k * sizeof(int));
    return new_node;
}

void insert_rec(kd_tree_t *kd, int *point, kd_node_t *current_node, int depth) {
    if (point[depth % kd->k] < current_node->data[depth % kd->k]) {
        if (current_node->left)
            insert_rec(kd, point, current_node->left, depth + 1);
        else
            current_node->left = get_node(point, kd->k);
    } else {
        if (current_node->right)
            insert_rec(kd, point, current_node->right, depth + 1);
        else
            current_node->right = get_node(point, kd->k);
    }
}

void insert(kd_tree_t *kd, int *point) {
    if (kd->root == NULL)
        kd->root = get_node(point, kd->k);
    else
        insert_rec(kd, point, kd->root, 0);
}

void load(kd_tree_t *kd, char *file_name) {
    FILE *fin = fopen(file_name, "rt");
    fscanf(fin, "%d %d", &kd->n, &kd->k);

    int **a = alloc(kd->n, kd->k);
    for (int i = 0; i < kd->n; ++i) {
        for (int j = 0; j < kd->k; ++j) {
            fscanf(fin, "%d", &a[i][j]);
        }
    }

    fclose(fin);
    shuffle(a, kd->n);
    for (int i = 0; i < kd->n; ++i)
        insert(kd, a[i]);

    dealloc(a, kd->n);
}

void nearest_neighbour_helper(kd_tree_t *kd, kd_node_t *current_node, int *point, int ***neighbours, int *cnt, int *current_distance, int depth) {
    if (point[depth % kd->k] < current_node->data[depth % kd->k]) {
        if (current_node->left) {
            nearest_neighbour_helper(kd, current_node->left, point, neighbours, cnt, current_distance, depth + 1);
        }

        if (current_node->right && sqr(point[depth % kd->k] - current_node->data[depth % kd->k]) <= *current_distance) {
            nearest_neighbour_helper(kd, current_node->right, point, neighbours, cnt, current_distance, depth + 1);
        }
    } else {
        if (current_node->right) {
            nearest_neighbour_helper(kd, current_node->right, point, neighbours, cnt, current_distance, depth + 1);
        }

        if (current_node->left && sqr(point[depth % kd->k] - current_node->data[depth % kd->k]) <= *current_distance) {
            nearest_neighbour_helper(kd, current_node->left, point, neighbours, cnt, current_distance, depth + 1);
        }
    }

    int dist = (*kd->dist_fun)(point, current_node->data, kd->k);
    if (*current_distance > dist) {
        *current_distance = dist;
        *cnt = 1;
        *neighbours = (int **)realloc(*neighbours, (*cnt) * sizeof(int *));
        (*neighbours)[*cnt - 1] = current_node->data;
    } else if (*current_distance == dist) {
        *cnt = *cnt + 1;
        *neighbours = (int **)realloc(*neighbours, (*cnt) * sizeof(int *));
        (*neighbours)[*cnt - 1] = current_node->data;
    }
}

void nearest_neighbour(kd_tree_t *kd, int *point) {
    if (!kd->root)
        return;

    int **neighbours = NULL, distance = INF, cnt = 0;
    nearest_neighbour_helper(kd, kd->root, point, &neighbours, &cnt, &distance, 0);
    mergesort(neighbours, kd->k, 0, cnt);
    for (int i = 0; i < cnt; ++i) {
        for (int j = 0; j < kd->k; ++j)
            printf("%d ", neighbours[i][j]);
        printf("\n");
    }

    free(neighbours);
}

int between(int *point, int *left, int *right, int k) {
    for (int i = 0; i < k; ++i)
        if (point[i] < left[i] || point[i] > right[i])
            return 0;

    return 1;
}

void range_search_helper(kd_tree_t *kd, kd_node_t *current_node, int *left, int *right, int ***ans, int *cnt, int depth) {
    if (between(current_node->data, left, right, kd->k)) {
        *cnt = *cnt + 1;
        *ans = (int **)realloc(*ans, (*cnt) * sizeof(int *));
        DIE(!(*ans), "realloc");

        (*ans)[*cnt - 1] = current_node->data;
    }

    if (left[depth % kd->k] < current_node->data[depth % kd->k] && current_node->left) {
        range_search_helper(kd, current_node->left, left, right, ans, cnt, depth + 1);
    }

    if (right[depth % kd->k] >= current_node->data[depth % kd->k] && current_node->right) {
        range_search_helper(kd, current_node->right, left, right, ans, cnt, depth + 1);
    }
}

void range_search(kd_tree_t *kd, int *left, int *right) {
    int **ans = NULL, cnt = 0;
    if (!kd->root)
        return;

    range_search_helper(kd, kd->root, left, right, &ans, &cnt, 0);
    mergesort(ans, kd->k, 0, cnt);
    for (int i = 0; i < cnt; ++i) {
        for (int j = 0; j < kd->k; ++j)
            printf("%d ", ans[i][j]);
        printf("\n");
    }
    free(ans);
}

void free_all_helper(kd_node_t *node) {
    if (node->left)
        free_all_helper(node->left);

    if (node->right)
        free_all_helper(node->right);

    free(node->data);
    free(node);
}

void free_all(kd_tree_t *kd) {
    if (!kd->root)
        return;

    free_all_helper(kd->root);
}