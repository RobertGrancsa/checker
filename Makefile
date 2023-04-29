CC=gcc
CFLAGS=-g

build: test

test: test.o
	$(CC) $(CFLAGS) $^ -o $@

%.o: %.c
	$(CC) $(CFLAGS) -c $^ -o $@

clean:
	rm -rf test *.o