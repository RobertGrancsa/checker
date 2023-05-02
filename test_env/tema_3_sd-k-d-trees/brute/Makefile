# compiler setup
CC=gcc
CFLAGS=-Wall -Wextra -Wshadow -Wpedantic -std=c99 -O3

# define targets
TARGETS=brute

#define dependencies
DEPS=matrix.h utils.h

#define object-files
OBJ=main.o matrix.o utils.o

build: $(TARGETS)

brute: $(OBJ)
	$(CC) $(CFLAGS) *.o -o brute

%.o: %.c $(DEPS)
	$(CC) $(CFLAGS) -c -o $@ $<

pack:
	zip -FSr 311CA_MarcelPetrescu_Tema3.zip README.md Makefile *.c *.h

clean:
	rm -f $(TARGETS) $(OBJ)

.PHONY: pack clean
