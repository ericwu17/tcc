int main() {
    int total  = 0;

    char a = 0;
    while (a != -1) {
        total += 1;
        a ++;
    }

    return total == 255 && a == -1;
}
