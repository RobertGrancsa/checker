#pragma once

#include <stdbool.h>

#define ALPHABET_SIZE 26

typedef struct trie trie_t;

struct trie {
  trie_t *children[ALPHABET_SIZE];
  int freq, n_children;
  bool is_end;
};

trie_t *create_trie();
void trie_insert(trie_t *trie, const char *key);
void trie_remove(trie_t *trie, const char *key);
const trie_t *trie_search(const trie_t *trie, const char *key);
void trie_free(trie_t *trie);
