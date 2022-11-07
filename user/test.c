#include <stdint.h>

extern void putchar(char c);

static char *hw = "Hello world!\n";

void putint(uint64_t n) {
  char buf[32] = {};
  int top = 0;
  while(n != 0) {
    buf[top] = (n % 10) + '0';
    n /= 10;
    ++top;
  }

  if(top == 0) {
    buf[0] = '0';
  } else --top;

  for(; top >= 0; --top) putchar(buf[top]);
  putchar('\n');
}

void _start() {
  for(uint64_t i = 0;; ++i) putint(i);
}
