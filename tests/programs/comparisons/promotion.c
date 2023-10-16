int main() {
    char a = 50;
    char b = 50;

    // should evaluate to true, since a and b both get "promoted" to ints.
    return a * b == 2500;
}
