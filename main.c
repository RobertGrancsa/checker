#include "trie.h"
#include "utils.h"
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/queue.h>

struct elem {
  char word[MAX_WORD_LEN + 1];
  const trie_t *trie;
} queue[10000000];

bool find_first_dfs(const trie_t *trie, char *buf) {
  if (!trie)
    return false;

  if (trie->is_end) {
    *buf = '\0';
    return true;
  }

  for (int i = 0; i < ALPHABET_SIZE; ++i)
    if (trie->children[i] != NULL) {
      *buf = 'a' + i;

      if (find_first_dfs(trie->children[i], buf + 1))
        return true;
    }

  return false;
}

bool find_first_bfs(const trie_t *trie, char *buffer) {
  if (trie == NULL)
    return false;
  printf("got here\n");

  int write = 1, read = 0;

  queue[0].word[0] = '\0';
  queue[0].trie = trie;

  while (read != write) {
    struct elem current = queue[read++];

    if (current.trie->is_end) {
      strcpy(buffer, current.word);
      return true;
    }

    for (int i = 0; i < ALPHABET_SIZE; ++i)
      if (current.trie->children[i] != NULL) {
        int len = strlen(current.word);

        strcpy(queue[write].word, current.word);
        queue[write].word[len] = 'a' + i;
        queue[write].word[++len] = '\0';
        queue[write++].trie = current.trie->children[i];
      }
  }

  return false;
}

void find_max_freq_dfs(const trie_t *trie, char *buf, int buf_len, char *best,
                       int *max_freq) {
  if (!trie)
    return;

  if (trie->is_end)
    if (trie->freq > *max_freq) {
      strcpy(best, buf);
      *max_freq = trie->freq;
    }

  for (int i = 0; i < ALPHABET_SIZE; ++i)
    if (trie->children[i] != NULL) {
      buf[buf_len] = 'a' + i;

      find_max_freq_dfs(trie->children[i], buf, buf_len + 1, best, max_freq);
    }
}

void autocomplete1(const trie_t *trie, const char *prefix) {
  static char buffer[MAX_WORD_LEN + 1];
  memset(buffer, 0, MAX_WORD_LEN + 1);

  if (find_first_dfs(trie_search(trie, prefix), buffer))
    printf("%s%s\n", prefix, buffer);
  else
    puts("No words found");
}

void autocomplete2(const trie_t *trie, const char *prefix) {
  static char buffer[MAX_WORD_LEN + 1];
  memset(buffer, 0, MAX_WORD_LEN + 1);

  if (find_first_bfs(trie_search(trie, prefix), buffer))
    printf("%s%s\n", prefix, buffer);
  else
    puts("No words found");
}

void autocomplete3(const trie_t *trie, const char *prefix) {
  static char buffer[MAX_WORD_LEN + 1], best[MAX_WORD_LEN + 1];
  memset(buffer, 0, MAX_WORD_LEN + 1);
  memset(best, 0, MAX_WORD_LEN + 1);
  int max_freq = -1;

  find_max_freq_dfs(trie_search(trie, prefix), buffer, 0, best, &max_freq);

  if (max_freq != -1)
    printf("%s%s\n", prefix, best);
  else
    puts("No words found");
}

bool autocorrect_dfs(const trie_t *trie, char *buf, int buf_len,
                     const char *word, int k) {
  // printf("|%*s|%s|%d|%d|\n", buf_len, buf, word, buf_len, k);
  if (k < 0)
    return false;

  if (*word == '\0') {
    if (trie->is_end) {
      printf("%*s\n", buf_len, buf);
      return true;
    }

    return false;
  }

  bool ret = false;

  for (int i = 0; i < ALPHABET_SIZE; ++i)
    if (trie->children[i] != NULL) {
      buf[buf_len] = 'a' + i;
      if (autocorrect_dfs(trie->children[i], buf, buf_len + 1, word + 1,
                          k - (*word != 'a' + i)))
        ret = true;
    }

  return ret;
}

void autocorrect(const trie_t *trie, const char *word, int k) {
  static char buffer[BUFSIZ];
  memset(buffer, 0, BUFSIZ);

  if (!autocorrect_dfs(trie, buffer, 0, word, k))
    puts("No words found");
}

void load_file(trie_t *trie, const char *file_name) {
  FILE *file = fopen(file_name, "r");
  DIE(file == NULL, "fopen");

  char word[MAX_WORD_LEN + 1];

  while (fscanf(file, "%s", word) != EOF)
    trie_insert(trie, word);

  fclose(file);
}

int main() {
  char line[BUFSIZ], command[BUFSIZ], arg1[BUFSIZ], arg2[BUFSIZ];
  trie_t *trie = create_trie();

  while (1) {
    fgets(line, BUFSIZ, stdin);
    sscanf(line, "%s%s%s", command, arg1, arg2);

    if (!strcmp(command, "INSERT"))
      trie_insert(trie, arg1);
    else if (!strcmp(command, "LOAD"))
      load_file(trie, arg1);
    else if (!strcmp(command, "REMOVE"))
      trie_remove(trie, arg1);
    else if (!strcmp(command, "AUTOCORRECT"))
      autocorrect(trie, arg1, atoi(arg2));
    else if (!strcmp(command, "AUTOCOMPLETE")) {
      if (!strcmp(arg2, "0")) {
        autocomplete1(trie, arg1);
        autocomplete2(trie, arg1);
        autocomplete3(trie, arg1);
      } else if (!strcmp(arg2, "1"))
        autocomplete1(trie, arg1);
      else if (!strcmp(arg2, "2"))
        autocomplete2(trie, arg1);
      else if (!strcmp(arg2, "3"))
        autocomplete3(trie, arg1);
    } else if (!strcmp(command, "EXIT"))
      break;
  }

  trie_free(trie);
}
