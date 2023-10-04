int main() {
    int a = 3;
    a || (a = 0) || (a = 4);
    return a;
}