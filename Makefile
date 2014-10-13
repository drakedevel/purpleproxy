CFLAGS = -fPIC -I/home/adrake/osrc/pidgin-main/libpurple `pkg-config --cflags glib-2.0`
OBJS = init.o stubs.o

purpleproxy.so: $(OBJS)
	gcc -o $@ -shared $(LDFLAGS) -ldl $^

stubs.c: genstub.py stubs.yaml
	python genstub.py

clean:
	$(RM) purpleproxy.so $(OBJS)

