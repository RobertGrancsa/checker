# compiler setup
CC=gcc
CFLAGS=-Wall -Wextra -Wshadow -Wpedantic -std=c99 -O3 -g

# define targets
TARGETS=kNN mk

#define dependencies
DEPS=k-d.h matrix.h utils.h

#define object-files
OBJ=k-d.o main_kd.o matrix.o utils.o trie.o main.o

build: $(TARGETS)

# mk: $(OBJ)
# 	$(CC) $(CFLAGS) main.o trie.o -o $@
mk:
	g++ main.cpp -o $@ -Ofast

kNN: $(OBJ)
	$(CC) $(CFLAGS) main_kd.o utils.o matrix.o k-d.o -o $@

%.o: %.c $(DEPS)
	$(CC) $(CFLAGS) -c -o $@ $<

pack:
	zip -FSr 311CA_MarcelPetrescu_Tema3.zip README.md Makefile *.c *.h

clean:
	rm -f $(TARGETS) $(OBJ)

.PHONY: pack clean
