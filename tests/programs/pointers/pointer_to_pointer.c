int main() {
    int x = 12;

    int * p = &x;
    int ** p = &p;

    return **p;
}