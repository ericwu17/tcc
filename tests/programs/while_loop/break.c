int main() {
    int sum = 0;
    int i = 0;
    while (i < 10) {
        sum = sum + i;
        if (sum > 10)
            break;

        i = i + 1;
    }
    return sum;
}