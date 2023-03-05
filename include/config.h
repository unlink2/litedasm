#ifndef CONFIG_H_
#define CONFIG_H_ 

#include "error.h"
#include "types.h"

struct Config {
  enum Error err;
};

extern struct Config cfg;

struct Config config_init();

#endif 
