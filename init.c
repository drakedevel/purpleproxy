#include <dlfcn.h>
#include <stdio.h>

#include "purple.h"
#include "init.h"

static int initialized = 0;
void *purple_handle;
FILE *purple_logf;

void purpleproxy_init()
{
    if (initialized)
        return;
    initialized = 1;
    char buf[128];
    sprintf(buf, "purple%d.log", getpid());
    purple_logf = fopen(buf, "w+");
}
