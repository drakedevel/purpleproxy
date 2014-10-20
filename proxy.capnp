@0xd244522a5701d934;

struct Plugin {
}

interface Mockingbird {
    purplePluginsGetProtocols @0 () -> (plugins :List(Plugin));
}