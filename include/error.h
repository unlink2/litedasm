#ifndef ERROR_H_
#define ERROR_H_ 

#define ERR_EXIT_ON_PANIC 

enum Error {
  OK = 0,
  ERR_FILE_NOT_FOUND,
  ERR_NOT_IMPLEMENTED
};

enum Warning {
  WARN_TEST
};

// bail on error macro
#define ok_or(err, ret) {if ((err)) { return (ret); }}

#ifdef ERR_EXIT_ON_PANIC 
#define panic(err, ...) { fprintf(stderr, __VA_ARGS__); exit(err); } // NOLINT
#else 
#define panic(err, ...) { fprintf(stderr, __VA_ARGS__); } // NOLINT
#endif 

#define info(...) { fprintf(stdout, __VA_ARGS__); }
#define warn(warn, ...) { fprintf(stderr, __VA_ARGS__); }
#define err(err, ...) { fprintf(stderr, __VA_ARGS__); }

#endif 
