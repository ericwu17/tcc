int main() {
  int primeCount = 0;

  int i = 2;
  
  while (i < 100) {    
    int isPrime = 1;
    int factor = 2;
    while (factor < i) {
      if (i % factor == 0) {
        isPrime = 0;
        factor = i; // this is our poor man's break
      }
      factor = factor + 1;
    }
    
    if (isPrime) {
      primeCount = primeCount + 1;
    }
    i = i + 1;
  }
  return primeCount;
}