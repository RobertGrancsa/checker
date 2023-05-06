#ifndef K_D_H_
#define K_D_H_

typedef struct kd_node_t kd_node_t;

struct kd_node_t {
	int *data;
	kd_node_t *left, *right;
};

typedef struct kd_tree_t kd_tree_t;

struct kd_tree_t {
	kd_node_t *root;
	int n, k;
	int (*dist_fun)(int *, int *, int);
};

void load(kd_tree_t *kd, char *file_name);
void nearest_neighbour(kd_tree_t *kd, int *point);
void range_search(kd_tree_t *kd, int *left, int *right);
void free_all(kd_tree_t *kd);

#endif
