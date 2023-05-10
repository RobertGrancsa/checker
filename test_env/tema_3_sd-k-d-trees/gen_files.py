from random import randint

# 10 tests
ns = [5, 10, 20, 30, 50, 100, 1000, 10000, 100000, 400000] # the number of points
ks = [2, 2, 2, 2, 2, 3, 3, 3, 3, 3] # the size of each point
qs = [2 * x for x in ns] # number of queries
intervals = [[-10, 10], [-100, 100], [-100, 100], [-100, 100], [-100, 100], [-10000, 10000], [-10000, 10000], [-10000, 10000], [-10000, 10000], [-10000, 10000]] # intervals for points

# generate data sets (load files)
for test in range(10):
    n = ns[test]
    k = ks[test]
    interval = intervals[test]

    name = "data/kNN" + str(test) + ".txt"
    f = open(name, "w")
    f.write(str(n) + " " + str(k) + "\n")
    for i in range(n):
        aux = ""
        for j in range(k):
            aux += str(randint(interval[0], interval[1])) + " "

        f.write(aux + "\n")

    f.close()

# generate queries (input files)
for test in range(10):
    q = qs[test]
    k = ks[test]
    interval = intervals[test]

    data_set_name = "data/kNN" + str(test) + ".txt"
    name = f"in/{test:02d}-kNN.in"
    f = open(name, "w")
    f.write("LOAD " + data_set_name + "\n")
    for i in range(q // 2):
        aux = ""
        for j in range(k):
            aux += str(randint(interval[0], interval[1])) + " "

        f.write("NN " + aux + "\n")

        aux = ""
        for j in range(k):
            n1 = randint(interval[0], interval[1])
            aux += str(n1) + " "
            aux += str(randint(n1, min(n1 + 20, interval[1]))) + " "

        f.write("RS " + aux + "\n")

    f.write("EXIT\n")
    f.close()