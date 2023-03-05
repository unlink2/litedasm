#include "config.h"
#include <string.h>

struct Config cfg;

struct Config config_init() {
  struct Config c;
  memset(&c, 0, sizeof(c));

  return c;
}
