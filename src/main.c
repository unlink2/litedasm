/**
 * When built without test
 */

#ifndef TEST
#ifdef LITEDASM

/// only use main if binary
#if TYPE == bin

#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <argp.h>
#include "debug.h"

const char *argp_program_version = "litedasm 0.1";
const char *argp_program_bug_address = "<lukas@krickl.dev>";
static char doc[] = "";
static char args_doc[] = "";

enum LongOptions { SAMPLE_LONG };

static struct argp_option options[] = {
    {"test", 'T', "VAR", 0, "Sampel command "}, {0}};

static error_t parse_opt(int key, char *arg,
                         struct argp_state *state) { // NOLINT
  // Config *c = state->input;
  switch (key) {
    break;
  case ARGP_KEY_ARG:
    if (state->arg_num > 0) {
      // Too many arguments
      argp_usage(state); // NOLINT
    } else {
      // TODO handle args
    }
    break;
  case ARGP_KEY_END:
    if (state->arg_num < 0) {
      /* Not enough arguments. */
      argp_usage(state); // NOLINT
    }
    break;
  default:
    return ARGP_ERR_UNKNOWN;
  }
  return 0;
}

static struct argp argp = {options, parse_opt, args_doc, doc};

int main(int argc, char **argv) {
  cfg = config_init();
  argp_parse(&argp, argc, argv, 0, 0, &cfg); // NOLINT
  return cfg.err;
}

#endif
#endif
#endif

/**
 * When built with test
 */
#ifdef TEST

#include "test.h"

int main(int argc, char **argv) {
  const struct CMUnitTest tests[] = {NULL};
  return cmocka_run_group_tests(tests, NULL, NULL);
}

#endif
