#include <cstdio>

#include <capnp/ez-rpc.h>

#include "proxy.capnp.h"

class MockingbirdImpl : public Mockingbird::Server {
    kj::Promise<void> purplePluginsGetProtocols(PurplePluginsGetProtocolsContext context) override;
};

kj::Promise<void> MockingbirdImpl::purplePluginsGetProtocols(PurplePluginsGetProtocolsContext context) {
    printf("Got purple_plugins_get_protocols\n");
    return kj::READY_NOW;
}

int main(int argc, char *argv[])
{
    // Set up a RPC server
    capnp::EzRpcServer server(argv[1]);
    server.exportCap("mockingbird", kj::heap<MockingbirdImpl>());

    // Wait forever
    auto& waitScope = server.getWaitScope();
    kj::NEVER_DONE.wait(waitScope);
}
