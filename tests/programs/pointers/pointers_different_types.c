int main() {
    // this program should not print anything to stdout
    // this program depends on the native machine having little-endian memory layout,
    // which is true for what I'm targeting.

    int arr[5] = {12,2,3,6,7};

    int* p = arr;
    char* cp = p;

    if (*p != 12) {
      putchar('a');
    }
    if (*(p+1) != 2) {
      putchar('a');
    }


    if (*cp != 12) {
      putchar('a');
    }
    if (*(cp+1) != 0) {
      putchar('a');
    }
    if (*(cp+2) != 0) {
      putchar('a');
    }
    if (*(cp+3) != 0) {
      putchar('a');
    }
    if (*(cp+4) != 2) {
      putchar('a');
    }

    if (*(cp+8) != 3) {
      putchar('a');
    }

    if (*(cp+12) != 6) {
      putchar('a');
    }
}