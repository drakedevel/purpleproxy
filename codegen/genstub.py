import json

from .config import StubConfig

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

def _to_capn_name(name):
    i = 0
    result = []
    upper = False
    while i < len(name):
        if name[i] == '_':
            upper = True
        else:
            result.append(name[i].upper() if upper else name[i])
            upper = False
        i += 1
    return ''.join(result)

def _emit_log(f, func, prefix=''):
    f.write('fprintf(proxy.logf, "%s%%s\\n", "%s");' % (prefix, func['name']))

def _emit_passthru(f, func):
    f.write('typedef %s(*orig_type)(%s);' % (func['return'], _to_typlist(func['params'])))
    f.write('static orig_type orig = NULL;')
    f.write('if (!orig) { orig = (orig_type)dlsym(RTLD_NEXT, "%s"); }' % func['name'])
    params = ', '.join('p%d' % i for i in range(len(func['params'])))
    if func['return'] == 'void':
        f.write('orig(%s);' % params)
    else:
        f.write('return orig(%s);' % params)

def _emit_proxy(f, func):
    _emit_log(f, func, prefix='PROXYING ')
    f.write('auto& waitScope = proxy.rpcClient->getWaitScope();')
    f.write('auto request = proxy.mb.%sRequest();' % _to_capn_name(func['name']))
    for pi, param in enumerate(func['params']):
        p_name = _to_capn_name('_' + param['name'])
        f.write('request.set%s(p%d);' % (p_name, pi))
    f.write('request.send().wait(waitScope);')
    _emit_passthru(f, func)

def _emit_logger(f, func):
    _emit_log(f, func)
    _emit_passthru(f, func)

def main():
    with open('purple-protos/decls.json') as f:
        decls = json.load(f)
    config = StubConfig.load('stubs.yaml')
    with open('stubs.c++', 'w+') as f:
        f.write('#include <cstdio>\n')
        f.write('#include <dlfcn.h>\n')
        f.write('#include "init.h"\n')
        f.write('using purpleproxy::Proxy;\n')
        f.write('extern "C" {\n')
        f.write('#include "purple.h"\n')
        for func in sorted(decls['functions'], key=lambda k: k['name']):
            if func['variadic'] or config.skip(func['name']):
                if not config.skip(func['name']):
                    print "WARNING: Skipping variadic %s" % (func['name'],)
                continue

            f.write('%s %s(%s) {' % (func['return'], func['name'], _to_arglist(func['params'])))
            f.write('Proxy &proxy = Proxy::get();');
            if config.proxy(func['name']):
                _emit_proxy(f, func)
            else:
                _emit_logger(f, func)
            f.write('}\n')
        f.write('}\n')

if __name__ == '__main__':
    main()
