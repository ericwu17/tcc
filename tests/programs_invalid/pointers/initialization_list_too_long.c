int main() {
    // Note: gcc allows this program, but I decided that tcc should fail at compile time
    // for initialization lists that are too long.
    int arr[5] = {1,2,3,6, 12, 13};
    return arr[0];
}