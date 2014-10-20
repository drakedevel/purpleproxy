import json
import random
import sys

from .config import StubConfig
from .translation import Translator

class CapnpEmitter(object):
    def __init__(self, f):
        self._counter = 0
        self._f = f
        self._indent = 0
        self._emit_header()

    @staticmethod
    def _format_typenames(tns):
        return '(%s)' % (', '.join('%s :%s' % tn for tn in tns))

    def _line(self, s, *args):
        self._f.write('    ' * self._indent)
        self._f.write(s % args)
        self._f.write('\n')

    def _emit_header(self):
        self._line('@0x%016x;', random.randrange(2**63, 2**64))
        self._line('using T = import "types.capnp";')

    def begin_interface(self, name):
        self._line('interface %s {', name)
        self._counter = 0
        self._indent += 1

    def method(self, name, params, returns):
        self._line('%s @%d %s -> %s;' % (name, self._counter,
                                         self._format_typenames(params),
                                         self._format_typenames(returns)))
        self._counter += 1

    def end_interface(self):
        self._indent -= 1
        self._line('}')

def main():
    fname = sys.argv[1]
    configname = sys.argv[2]
    config = StubConfig.load(configname)
    with open(fname, 'r') as f:
        decls = Translator(config).process_decls(json.load(f))

    capnp = CapnpEmitter(sys.stdout)
    capnp.begin_interface('Mockingbird')
    for func in sorted(decls, key=lambda k: k.p_name):
        if not config.proxy(func.c_name):
            continue
        capnp.method(func.p_name,
                     [(p.p_name, p.p_type) for p in func.params],
                     [(r.p_name, r.p_type) for r in func.ret])
    capnp.end_interface()


if __name__ == '__main__':
    main()
