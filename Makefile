CXXFLAGS = -g -fPIC `pkg-config --cflags capnp capnp-rpc glib-2.0 purple`
LIBS = -ldl `pkg-config --libs capnp capnp-rpc glib-2.0`

SO_OBJS = init.o stubs.o proxy.capnp.o
SERVER_OBJS = server.o proxy.capnp.o

all: purpleproxy.so server

purpleproxy.so: $(SO_OBJS)
	g++ -o $@ -shared $(LDFLAGS) $(LIBS) $^

server: $(SERVER_OBJS)
	g++ -o $@ $(LDFLAGS) $(LIBS) $^

stubs.c++: genstub.py stubs.yaml
	python genstub.py

clean:
	$(RM) purpleproxy.so $(SO_OBJS)
	$(RM) server $(SERVER_OBJS)

%.capnp:

%.capnp.c++ %.capnp.h: %.capnp
	capnp compile -oc++ $<

%.o: %.c++
	$(CXX) -o $@ -c -std=c++11 $(CXXFLAGS) $<
