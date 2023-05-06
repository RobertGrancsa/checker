# Checker

This checker is in it's first iteration. If you found any bugs and have features you
would like me to add, or any QoL ideas, contact me and I'll try to implement them
as soon as I can.

What's different from all the other checkers out there is that it uses a GUI instead
of the old-school text based printing or results. It can run any test individually,
it shows a diff of all the results and most importantly it should/will run more
tasks at the same time.

![alt-text](https://i.imgur.com/3uqwCvs.png)

## How to use the checker

Run the by using either one of these commands. You must be in the same directory
as the other checker files, otherwise it won't work. For this checker to work,
you will need a terminal at least 52x24.

```bash
$ ./check

$ checker-tema-3-sd

# If you only want to use the text only version, add the following flag to the command

$ ./check --legacy
```

## Keybinds

To simplify the use of the checker, you can use the following keybinds:

- `r` - runs all the tests from both tasks
- `f` - runs only the failed tasks that either have crashed or they got 0 points
- `v` - enables or disables valgrind globally for the tests (valgrind is enabled
when the `Tests` window is highlighted in red)
- `c` - runs the coding style checker and shows a pop-up showing all the possible problems
- `ctrl+c` or `q` - exit the program

