int main() {
  // returns 24 (this is a pointer to an array that is 3 wide)
  long(* l)[3];
  return sizeof(l[0]);
}