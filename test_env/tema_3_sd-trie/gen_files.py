import random
import os
import string

random.seed(69420)

directory = "books"
inputfolder = "in"
letters = string.ascii_lowercase

inserts = [50, 500, 2000, 10000]
query_frecv = [5, 10, 50, 100]
final_querys = [10, 50, 100, 1000]
remove_frecv = [1000, 20, 100, 200]

def reformat_books():
    for filename in os.listdir(directory):
        filename = directory + "/" + filename
        print(filename)
        text_file = ""
        with open(filename, "r") as f:
            text_file = f.read()
            text_file = "".join(char.lower() for char in text_file if char.isalpha() or char.isspace())
        
        with open(filename, "w") as f:
            f.write(text_file)

def load_words():
    with open('words_alpha.txt') as word_file:
        valid_words = list(word_file.read().split())

    return valid_words

def make_book_test(id, file):
    list_of_words = []

    if id == 0:
        file.write(f"LOAD data/dracula.txt\n")
        list_of_words = list(set(books["books/dracula.txt"].split()))
    
    if id == 1:
        file.write(f"LOAD data/moby_dick.txt\n")
        list_of_words = list(set(books["books/moby_dick.txt"].split()))

    if id == 2:
        file.write(f"LOAD data/great_gatsby.txt\n")
        file.write(f"LOAD data/little_women.txt\n")

        list_of_words = list(set(books["books/little_women.txt"].split()))
        list_of_words.extend(list(set(books["books/great_gatsby.txt"].split())))

    if id == 3:
        file.write(f"LOAD data/great_gatsby.txt\n")
        file.write(f"LOAD data/little_women.txt\n")
        file.write(f"LOAD data/dracula.txt\n")
        file.write(f"LOAD data/moby_dick.txt\n")
        file.write(f"LOAD data/romeo_juliet.txt\n")

        list_of_words = list(set(books["books/little_women.txt"].split()))
        list_of_words.extend(list(set(books["books/great_gatsby.txt"].split())))
        list_of_words.extend(list(set(books["books/dracula.txt"].split())))
        list_of_words.extend(list(set(books["books/moby_dick.txt"].split())))
        list_of_words.extend(list(set(books["books/romeo_juliet.txt"].split())))
    
    for num in range(final_querys[id] * 2):
        if id != 0:
            prefix: str = list_of_words[random.randint(0, len(list_of_words) - 1)]
            while len(prefix) < 4:
                prefix = list_of_words[random.randint(0, len(list_of_words) - 1)]

            prefix = prefix[:random.randint(1, len(prefix) - 2)]
            file.write(f"AUTOCOMPLETE {prefix} { random.randint(0, 3) }\n")

        if id != 1:
            mistakes = random.randint(1, 2)
            mistakes_final = mistakes
            word = list(list_of_words[random.randint(0, len(list_of_words) - 1)])

            for i, letter in enumerate(word):
                if random.randint(1, 2) == 1 and mistakes != 0:
                    random_l = random.choice(letters)
                    while random_l == letter:
                        random_l = random.choice(letters)
                    word[i] = random_l
                    mistakes -= 1
                    if mistakes == 0:
                        break
            word = "".join(word)
            file.write(f"AUTOCORRECT {word} {mistakes_final}\n")
        
        if num % remove_frecv[id] == 0:
            word = ""
            if random.randint(0, 100) == 1:
                word = random.choice(words)
            else:
                word = random.choice(list_of_words)
            file.write(f"REMOVE {word}\n")



def make_small_test(id, list_of_words, file):
    if id == 2:
        file.write(f"LOAD data/moby_dick.txt\n")

    if id == 3:
        file.write(f"LOAD data/dracula.txt\n")

    for num in range(inserts[id]):
        file.write(f"INSERT {list_of_words[num]}\n")

        if num == 0:
            continue

        if num % query_frecv[id] == 0:
            if id != 0:
                prefix: str = list_of_words[random.randint(0, num)]
                while len(prefix) < 4:
                    prefix = list_of_words[random.randint(0, num)]
                prefix = prefix[:random.randint(1, len(prefix) - 2)]
                file.write(f"AUTOCOMPLETE {prefix} { random.randint(0, 3) }\n")

            if id != 1:
                mistakes = random.randint(1, 2)
                mistakes_final = mistakes
                word = list(list_of_words[random.randint(0, num)])

                for i, letter in enumerate(word):
                    if random.randint(1, 2) == 1 and mistakes != 0:
                        random_l = random.choice(letters)
                        while random_l == letter:
                            random_l = random.choice(letters)
                        word[i] = random_l
                        mistakes -= 1
                        if mistakes == 0:
                            break
                word = "".join(word)
                file.write(f"AUTOCORRECT {word} {mistakes_final}\n")

        if num % remove_frecv[id] == 0:
            word = ""
            if random.randint(0, 100) == 1:
                word = random.choice(words)
            else:
                word = list_of_words[random.randint(0, num)]
            file.write(f"REMOVE {word}\n")

	        

    if id == 3:
        file.write(f"LOAD data/romeo_juliet.txt\n")

    for _ in range(final_querys[id]):
        if id != 0:
            prefix: str = list_of_words[random.randint(0, num)]
            while len(prefix) < 4:
                prefix = list_of_words[random.randint(0, len(list_of_words) - 1)]
            prefix = prefix[:random.randint(1, len(prefix) - 1)]
            file.write(f"AUTOCOMPLETE {prefix} { random.randint(0, 3) }\n")

        if id != 1:
            mistakes = random.randint(1, 2)
            mistakes_final = mistakes
            word = list(list_of_words[random.randint(0, num)])

            for i, letter in enumerate(word):
                if random.randint(1, 2) == 1 and mistakes != 0:
                    while random_l == letter:
                        random_l = random.choice(letters)
                    word[i] = random_l
                    mistakes -= 1
                    if mistakes == 0:
                        break
            word = "".join(word)
            file.write(f"AUTOCORRECT {word} {mistakes_final}\n")



words = load_words()
books = {}

for filename in os.listdir(directory):
    filename = directory + "/" + filename
    
    book = ""
    with open(filename, "r") as f:
        book = f.read()

    books[filename] = book

for i in range(4):
    index = i*2 + 2
    for _ in range(2):
        filename = f"{inputfolder}/{index:02d}-mk.in"
        file = open(filename, "w")

        if index % 2 == 0:
            inserted_words = []

            word_range = words[int(len(words)/64) * i*8:int(len(words)/64) * (i*8 + 1)]

            for _ in range(inserts[i]):
                inserted_words.append(random.choice(word_range))

            # print(inserted_words)s
            make_small_test(i, inserted_words, file)
        else:
            make_book_test(i, file)

        file.write("EXIT\n")
        file.close()
        index += 1
        


# reformat_books()



# 10 tests
# ns = [5, 10, 20, 30, 50, 100, 1000, 10000, 150000, 500000] # the number of points
# ks = [2, 2, 2, 2, 2, 3, 3, 3, 3, 3] # the size of each point
# qs = [2 * x for x in ns] # number of queries
# intervals = [[-10, 10], [-100, 100], [-100, 100], [-100, 100], [-100, 100], [-10000, 10000], [-10000, 10000], [-10000, 10000], [-10000, 10000], [-10000, 10000]] # intervals for points

# # generate data sets (load files)
# for test in range(10):
#     n = ns[test]
#     k = ks[test]
#     interval = intervals[test]

#     name = "data/kNN" + str(test) + ".txt"
#     f = open(name, "w")
#     f.write(str(n) + " " + str(k) + "\n")
#     for i in range(n):
#         aux = ""
#         for j in range(k):
#             aux += str(randint(interval[0], interval[1])) + " "

#         f.write(aux + "\n")

#     f.close()

# # generate queries (input files)
# for test in range(10):
#     q = qs[test]
#     k = ks[test]
#     interval = intervals[test]

#     data_set_name = "data/kNN" + str(test) + ".txt"
#     name = f"in/{test:02d}-kNN.in"
#     f = open(name, "w")
#     f.write("LOAD " + data_set_name + "\n")
#     for i in range(q // 2):
#         aux = ""
#         for j in range(k):
#             aux += str(randint(interval[0], interval[1])) + " "

#         f.write("NN " + aux + "\n")

#         aux = ""
#         for j in range(k):
#             n1 = randint(interval[0], interval[1])
#             aux += str(n1) + " "
#             aux += str(randint(n1, min(n1 + 20, interval[1]))) + " "

#         f.write("RS " + aux + "\n")

#     f.write("EXIT\n")
#     f.close()