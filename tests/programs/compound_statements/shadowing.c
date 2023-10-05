int main() {
  int foo = 0;
  {
      foo = 3; //changes outer foo
      int foo = 4; //defines inner foo, shadowing outer foo
  }
  return foo; //returns 3
}