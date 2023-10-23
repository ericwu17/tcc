int main() {
    // Note: gcc allows this, but tcc shall fail at compile time
    // since arrays of size 0 makes no sense
    int array[0];
    return array[0];
}