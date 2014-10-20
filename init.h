#ifndef _PURPLEPROXY_INIT_H
#define _PURPLEPROXY_INIT_H

#include <memory>

#include <capnp/ez-rpc.h>

#include "purpleproxy.h"
#include "proxy.capnp.h"

_PURPLEPROXY_NS_BEGIN

class Proxy {
private:
    static Proxy *instance;

    Proxy();

public:
    FILE *logf;
    capnp::EzRpcClient *rpcClient;
    Mockingbird::Client mb;

    static Proxy& get();
};

_PURPLEPROXY_NS_END

#endif
