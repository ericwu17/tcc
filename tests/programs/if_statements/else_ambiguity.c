int main() {
  // gcc will parse this statement as follows and then
  // return 0 (the else is joined to the second if)

  // gcc will emit a warning (-Wdangling-else) for this situation.
  int b = 0;
  if (0)
    if (1)
      b = 3;
    else
      b = 1;

  return b;
}