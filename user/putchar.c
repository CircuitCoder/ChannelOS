#include <stdint.h>
#include <stdatomic.h>

struct putchar_queue {
  volatile uint32_t recv;
  volatile uint32_t send;
  volatile uint8_t sleep;
  volatile uint8_t closed;

  volatile struct {
    uint64_t ident;
    uint64_t arg;
  } data[255];
};

void _start(struct putchar_queue *q, volatile uint8_t *serial) {
  while(1) {
    uint32_t cur = atomic_load_explicit(&q->recv, memory_order_relaxed);
    if(atomic_load_explicit(&q->send, memory_order_acquire) > cur) {
      uint64_t arg = q->data[cur % 255].arg;
      *serial = (uint8_t) arg;
      atomic_store(&q->recv, cur + 1);
    }
  }
}
