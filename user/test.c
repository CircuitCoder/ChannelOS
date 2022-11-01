#include <stdint.h>

extern uint64_t kernel_meow();

void _start() {
  kernel_meow();
}
