#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

int main(void)
{
    char *a = 0x0;
    char *ceva = malloc(10000);
    printf("Testing\n");

    scanf("%s", ceva);


    if (strcmp(ceva, "dasdasd") == 0)
        *a = 5;

    sleep(2);
    
    printf("%s\n", ceva);
    printf("Testing\n");
}