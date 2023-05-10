#include "trie.h"
#include "utils.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

trie_t *create_trie() {
  trie_t *trie = malloc(sizeof(*trie));
  DIE(trie == NULL, "malloc");

  memset(trie->children, 0, sizeof(trie->children));
  trie->freq = trie->n_children = 0;
  trie->is_end = false;

  return trie;
}

void trie_insert(trie_t *trie, const char *key) {
  if (*key == '\0') {
    ++trie->freq;
    trie->is_end = true;
    return;
  }

  if (trie->children[*key - 'a'] == NULL) {
    trie->children[*key - 'a'] = create_trie();
    ++trie->n_children;
  }

  trie_insert(trie->children[*key - 'a'], key + 1);
}

void trie_remove(trie_t *trie, const char *key) {
  if (*key == '\0') {
    trie->freq = 0;
    trie->is_end = false;
    return;
  }

  if (trie->children[*key - 'a'] == NULL)
    return;

  trie_remove(trie->children[*key - 'a'], key + 1);

  if (trie->children[*key - 'a']->n_children == 0) {
    free(trie->children[*key - 'a']);
    trie->children[*key - 'a'] = NULL;
    --trie->n_children;
  }
}

const trie_t *trie_search(const trie_t *trie, const char *key) {
  if (*key == '\0')
    return trie;

  if (trie->children[*key - 'a'] == NULL)
    return NULL;

  return trie_search(trie->children[*key - 'a'], key + 1);
}

void trie_free(trie_t *trie) {
  for (int i = 0; i < ALPHABET_SIZE; ++i)
    if (trie->children[i] != NULL)
      trie_free(trie->children[i]);

  free(trie);
}
