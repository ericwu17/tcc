int main() {
    int total = 0;
    int i = 0;
    while (i <= 10) {
        if (i %2 == 0) {
            i = i + 1;
            continue;
        }
        total = total + i;
        i = i + 1;
    }
    return total;

}