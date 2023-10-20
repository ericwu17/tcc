int print_a_bunch_of_nums(
    int arg1,
    int arg2,
    int arg3,
    int arg4,
    int arg5,
    int arg6,
    int arg7,
    int arg8,
    int arg9
) {
    printNum(arg1);
    printNum(arg2);
    printNum(arg3);
    printNum(arg4);
    printNum(arg5);
    printNum(arg6);
    printNum(arg7);
    printNum(arg8);
    printNum(arg9);
}

int main() {

    print_a_bunch_of_nums(55, 67, 83, 12, 77, 80, 17, 289, 4913);
}

int printNum(int num) {
    printNumHelper(num);
    putchar('\n');
}

int printNumHelper(int num) {
    if (num != 0) {
        printNumHelper(num / 10);
        putchar(num % 10 + '0');
    }
}