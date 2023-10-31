int main() {
    char buf[20] = "hello world\n";

    char* p = buf;
    while (*p != '\0'){
      putchar(*p);
      p++;
    }
}