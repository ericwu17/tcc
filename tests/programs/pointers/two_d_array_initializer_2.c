int main() {
  char arr[5][5] = {"abc", "def", "ghi"};

  for (int i = 0; i < 3; i ++) {
    for (int j = 0; j < 3; j ++) {
      putchar(arr[i][j]);
    }
  }
}