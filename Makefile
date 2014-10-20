CXXFLAGS = -g -fPIC `pkg-config --cflags capnp capnp-rpc glib-2.0 purple`
LIBS = -ldl `pkg-config --libs capnp capnp-rpc glib-2.0`

PROTOS = proxy.capnp types.capnp
PROTO_OBJS = $(PROTOS:.capnp=.capnp.o)
PROTO_HDRS = $(PROTOS:.capnp=.capnp.h)

SO_OBJS = init.o stubs.o $(PROTO_OBJS)
SERVER_OBJS = server.o $(PROTO_OBJS)

all: purpleproxy.so server

DEPS := $(SO_OBJS:.o=.d) $(SERVER_OBJS:.o=.d)
-include $(DEPS)

%.capnp:

%.capnp.c++ %.capnp.h: %.capnp
	capnp compile -oc++ $<

%.o: %.c++ $(PROTO_HDRS)
	$(CXX) $(CXXFLAGS) -c -std=c++11 -o $@ $<
	$(CXX) $(CXXFLAGS) -c -std=c++11 -MM -MF $(patsubst %.o,%.d,$@) $< 

purpleproxy.so: $(SO_OBJS)
	g++ -o $@ -shared $(LDFLAGS) $(LIBS) $^

server: $(SERVER_OBJS)
	g++ -o $@ $(LDFLAGS) $(LIBS) $^

stubs.c++: codegen/genstub.py stubs.yaml
	python2 -m codegen.genstub

proxy.capnp: codegen/gencapnp.py stubs.yaml
	python2 -m codegen.gencapnp purple-protos/decls.json stubs.yaml >$@

clean:
	$(RM) purpleproxy.so server
	$(RM) *.o *.d $(PROTO_HDRS)
