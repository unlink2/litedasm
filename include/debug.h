#ifndef DEBUG_H_
#define DEBUG_H_ 

#ifndef DEBUG  

#define debug(...) 
#define debug_assert(to_assert) 

#else 

#include "assert.h"

#define debug(...) fprintf(stderr, __VA_ARGS__); 
#define debug_assert(to_assert) assert((to_assert));

#endif 

#endif 
