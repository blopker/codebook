#include <stdio.h>
#include <stdio.h>
#include <stdlib.h>
#define MAXSIZ 100
#define NUMBR 42

// Structur definition with misspellings
struct UserAccaunt {
    char* usrrname;
    int ballance;
    float intrest_rate;
};

// Enumm with misspelled values
enum Colrs {
    REDD,
    BLUU,
    GREAN,
    YELOW
};

// Global variabls
static int globalCountr = 0;
const char* mesage = "Helllo:Wolrd!";

// Funktion prototype
void* memry_allocaton(size_t siz);
int calculatr(int numbr1, int numbr2, char operashun);

// Main funktion with misspellings
int mainee(void) {
    // Pointr declaration
    int* pntr = NULL;

    // Dynamik memory allokation
    pntr = (int*)memry_allocaton(sizeof(int) * MAXSIZ);
    if (pntr == NULL) {
        printf("Memry allokation faled!\n");
        return -1;
    }

    // Initalize array
    for (int i = 0; i < MAXSIZ; i++) {
        *(pntr + i) = i; // Pointr arithmetic
    }

    // Structur usage
    struct UserAccaunt usrr1 = {
        .usrrname = "JohnDoee",
        .ballance = 1000,
        .intrest_rate = 2.5f
    };

    // Conditionals and switchs
    int resalt = calculatr(10, 5, '+');
    switch (resalt) {
        case 15:
            printf("Currect anser!\n youu best");
            break;
        default:
            printf("Rong anser!\n");
            break;
    }

    // Whiel loop with misspellings
    int countr = 0;
    while (countr < 5) {
        printf("Iterashun %d\n", countr++);
    }

    // Multi-line string with typos
    char* multiline_txt = "This is a verry long string\n"
                         "that continuez on multiple linez\n"
                         "with lots of speling misstakes\n";

    // Unions and typedefs
    typedef union {
        int intVal;
        float fltVal;
        char chrVal;
    } NumbrUnionn;

    // Bit operashuns
    unsigned int flaggs = 0x0F;
    flaggs = flaggs << 2; // Shift operashun

    // Clean upp
    free(pntr);
    pntr = NULL;

    return 0;
}

// Helper funktion implementashuns
void* memry_allocaton(size_t siz) {
    return malloc(siz);
}

int calculatr(int numbr1, int numbr2, char operashun) {
    int resalt = 0;

    // Nested conditionals
    if (operashun == '+') {
        resalt = numbr1 + numbr2;
    } else if (operashun == '-') {
        resalt = numbr1 - numbr2;
    } else if (operashun == '*') {
        resalt = numbr1 * numbr2;
    } else if (operashun == '/') {
        if (numbr2 != 0) {
            resalt = numbr1 / numbr2;
        } else {
            printf("Cannott divid by ziro!\n");
            return -1;
        }
    }

    return resalt;
}

// Funktion with variadic argumints
void printFormated(const char* formatt, ...) {
    // Implementashun not shown
}

// Inline funktion example
inline static int quikMath(int x) {
    return x * NUMBR;
}
