int main() {
    char buf[20] = "hello world\n\0";

    char* p = buf;
    while (*p != '\0'){
      putchar(*p);
      p++;
    }
}