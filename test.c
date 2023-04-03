#include <stdio.h>
#include <stdlib.h>

int main(void)
{
    // char *a = 0x0;
    // *a = 5;
    char *ceva = malloc(10000);
    printf("Testing\n");

    scanf("%s", ceva);

    printf("%s\n", ceva);
    printf("Testing\n");
}