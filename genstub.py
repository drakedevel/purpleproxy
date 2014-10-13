#!/usr/bin/env python
import json
import yaml

def _to_decl(type_, name):
    if '(*)' in type_:
        return type_.replace('(*)', '(*%s)' % name)
    if type_.endswith('[]'):
        bits = type_.split(' ')
        return ' '.join(bits[:-1] + [name, bits[-1]])
    return '%s %s' % (type_, name)

def _to_arglist(types):
    return ', '.join(_to_decl(p['type'], 'p%d' % i) for i, p in enumerate(types))

def _to_typlist(types):
    return ','.join(p['type'] for p in types)

class StubConfig(object):
    def __init__(self, dct):
        self._dict = dct

    def skip(self, name):
        return (name in self._dict.get('skip', []) or
                name in self._dict.get('passthrough', []))

def main():
    with open('purple-protos/decls.json') as f:
        decls = json.load(f)
    with open('stubs.yaml') as f:
        config = StubConfig(yaml.safe_load(f.read()))
    with open('stubs.c', 'w+') as f:
        f.write('#define _GNU_SOURCE\n')
        f.write('#include "purple.h"\n')
        f.write('#include "init.h"\n')
        f.write('#include <stdio.h>\n')
        f.write('#include <dlfcn.h>\n')
        for func in sorted(decls['functions'], key=lambda k: k['name']):
            if config.skip(func['name']):
                continue
            if func['variadic']:
                print "WARNING: Skipping variadic %s" % (func['name'],)
                continue
            f.write('%s %s(%s) {' % (func['return'], func['name'], _to_arglist(func['params'])))
            f.write('purpleproxy_init();');
            f.write('static %s(*orig)(%s) = NULL;' % (func['return'], _to_typlist(func['params'])))
            f.write('if (!orig) { orig = dlsym(RTLD_NEXT, "%s"); }' % func['name'])
            params = ', '.join('p%d' % i for i in range(len(func['params'])))
            f.write('fprintf(purple_logf, "%%s\\n", "%s");' % func['name'])
            if func['return'] == 'void':
                f.write('orig(%s);' % params)
            else:
                f.write('return orig(%s);' % params)
            f.write('}\n')

if __name__ == '__main__':
    main()
