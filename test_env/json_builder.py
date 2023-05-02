import os
import json

directory = "tema_3_sd-k-d-trees"
test_path = directory + "/in"

test = []
i = 0
for filename in os.listdir(test_path):
    f = os.path.join(test_path, filename)
    # checking if it is a file
    if os.path.isfile(f):
        dictionary = {
            "id": i,
            "name": "kNN " + str(i + 1),
            "status": "0",
            "log": "",
            "time_normal": 0.0,
            "time_valgrind": 0.0,
            "timeout": 30,
            "test_score": int(45 / len(os.listdir(test_path)))
        }

        test.append(dictionary)
        i += 1
        print(f)

json_object = json.dumps(test, indent = 4)
print(json_object)

with open("tests.json", "w") as outfile:
    json.dump(test, outfile, indent = 2)

