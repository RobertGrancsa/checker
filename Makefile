# compiler setup
CC=gcc
CFLAGS=-Wall -Wextra -Wshadow -Wpedantic -std=c99 -O3

# define targets
TARGETS=kNN mk

#define dependencies
DEPS=k-d.h matrix.h utils.h

#define object-files
OBJ=k-d.o main.o matrix.o utils.o

build: $(TARGETS)

mk:
	$(CC) $(CFLAGS) test.c -o $@

kNN: $(OBJ)
	$(CC) $(CFLAGS) *.o -o $@

%.o: %.c $(DEPS)
	$(CC) $(CFLAGS) -c -o $@ $<

pack:
	zip -FSr 311CA_MarcelPetrescu_Tema3.zip README.md Makefile *.c *.h

clean:
	rm -f $(TARGETS) $(OBJ)

.PHONY: pack clean
