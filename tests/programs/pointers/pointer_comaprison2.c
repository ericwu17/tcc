int main() {
    int arr[5];
    arr[0] = 4;
    arr[1] = 5;
    arr[2] = 6;
    arr[3] = 12;
    arr[4] = 22;

    int *end_ptr = arr +5;

    int sum = 0;
    for (int* p = arr; p != end_ptr; p ++) {
      sum += *p;
    }

    return sum;

}