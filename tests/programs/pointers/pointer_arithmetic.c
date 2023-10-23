int main() {
    // this program should not print anything to stdout

    int arr[5] = {1,2,3,6,7};
    
    int* p = arr;
    
    if (*p != 1) {
      putchar('a');
    }
    p ++;
    if (*p != 2) {
      putchar('a');
    }
    if (*(p+1) != 3) {
      putchar('a');
    }
    if (p[1] != 3) {
      putchar('a');
    }

    p = p + 2;

    if (p[0] != 6) {
      putchar('a');
    }
    if (*p != 6) {
      putchar('a');
    }

    p += 1;

    if (*p != arr[4]) {
        putchar('a');
    }
    
}