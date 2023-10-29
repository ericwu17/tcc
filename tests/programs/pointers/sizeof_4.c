int main() {
  // returns 8 (this is a pointer to an array that is 3 wide)
  long(* l)[3];
  return sizeof(l);
}