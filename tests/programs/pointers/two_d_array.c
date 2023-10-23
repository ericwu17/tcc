int main() {
  // this program should exit with code 0

  // 2d arrays are layed out sequentially in memory.

  int arr[3][3];

  arr[0][0] = 3;
  arr[0][1] = 3;
  arr[0][2] = 3;

  arr[0][3] = 17;

  if (arr[1][0] != 17) {
    return 1;
  }
  if (arr[0][3] != 17) {
    return 1;
  }

  int* p = &arr[0][0];

  p[3] = 22;
  p[8] = 91;

  if (arr[1][0] != 22) {
    return 1;
  }
  if (arr[2][2] != 91) {
    return 1;
  }

 
}