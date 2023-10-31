int main() {
    int buf[5] = {1,2,3,4,5};

    int *ptr = buf+2;

    if (*(ptr+1) != 4)
      return 1;
    if (*(ptr-1) != 2)
      return 1;
    if (*(ptr-2) != 1)
      return 1;
    

}