#include <cstdio>
#include <dlfcn.h>
#include <unistd.h>

#include <capnp/ez-rpc.h>

#include "init.h"
#include "proxy.capnp.h"

extern "C" {
#include "purple.h"
}

using purpleproxy::Proxy;

Proxy *Proxy::instance = nullptr;

Proxy::Proxy() : mb(nullptr)
{
    // Initialize un-hooked call logging
    char buf[128];
    sprintf(buf, "purple%d.log", getpid());
    logf = fopen(buf, "w+");

    // Initialize CapnProto RPC client
    rpcClient = new capnp::EzRpcClient("localhost:1337");
    mb = rpcClient->importCap<Mockingbird>("mockingbird");
}

Proxy& Proxy::get()
{
    if (instance)
        return *instance;
    return *(instance = new Proxy());
}
