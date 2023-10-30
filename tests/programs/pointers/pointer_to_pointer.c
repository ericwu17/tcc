int main() {
    int x = 12;

    int * p = &x;
    int ** pp = &p;

    return **pp;
}