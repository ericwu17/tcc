int main() {
    int buf[20];

    int* p = buf;
    int* p2 = buf+12;
    
    if (p2 - p != 12) {
      return 1;
    }
    if (p - p2 != -12) {
      return 1;
    }
    
    
}