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

    if (strcmp(ceva, "asdasd") == 0)
        sleep(2);

    if (strcmp(ceva, "loop") == 0) {
        for (size_t i = 0; i < 50; i++) {
            printf("around the world\n");
        }
    }

    if (strcmp(ceva, "malloc") == 0) {
        
        char *alt = malloc(10000); 
    }
    
    
    printf("%s\n", ceva);
    printf("Testing\n");
    free(ceva);
}