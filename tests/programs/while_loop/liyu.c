int main() {
  int num = 32;
  while (num > 1) {
    if (num % 2 != 0){
      return 0;
    }
    else{
      num = num / 2;
    }
  }
  return 1;
}