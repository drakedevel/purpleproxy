import json

from ..config import StubConfig

def _to_rusttyp(type_):
    if '(*)' in type_:
        bits = type_.split(' ')
        args = ' '.join(bits[1:])[4:-1].split(',')
        return '*mut (|' + ', '.join([ _to_rusttyp(t) for t in args ]) + '| -> ' + _to_rusttyp(bits[0]) + ')'
    if type_.endswith('*'):
        bits = type_.split(' ')
        if bits[0] == 'const':
            return '*const ' + _to_rusttyp(' '.join(bits[1:-1]))
        else:
            return '*mut ' + _to_rusttyp(' '.join(bits[0:-1]))
    if type_.endswith('[]'):
        bits = type_.split(' ')
        return _to_rusttyp(' '.join(bits[:-1]) + ' *')
    if type_.startswith('unsigned'):
        bits = type_.split(' ')
        return _to_rusttyp('u' + ' '.join(bits[1:]))
    if type_ in ['long', 'ulong',
                 'int', 'uint',
                 'short', 'ushort',
                 'char', 'uchar',
                 'void']:
        return 'c_' + type_
    return type_

def _to_decl(type_, name):
    return '%s : %s' % (name, _to_rusttyp(type_))

def _to_arglist(types):
    return ', '.join(_to_decl(p['type'], 'p%d' % i) for i, p in enumerate(types))

def _to_typlist(types):
    return ', '.join(_to_rusttyp(p['type']) for p in types)

def _emit_log(f, func, prefix=''):
    f.write('        writeln!(&mut proxy.log, "%s{}", "%s");\n' % (prefix, func['name']))

def _emit_passthru(f, func):
    f.write('        unsafe {\n')
    f.write('            static mut orig: *mut |%s| -> %s = ::std::ptr::null();\n' \
            % (_to_typlist(func['params']), _to_rusttyp(func['return'])))
    f.write('            if orig.is_null() { orig = dlsym(RTLD_NEXT, "%s"); }\n' % func['name'])
    params = ', '.join('p%d' % i for i in range(len(func['params'])))
    f.write('            (*orig)(%s)\n' % params)
    f.write('        }\n')

def _emit_proxy(f, func):
    _emit_log(f, func, prefix='PROXYING ')
    f.write('        let waitScope = proxy.rpcClient.get_wait_scope();\n')
    f.write('        let request = proxy.mb.%s_request();\n' % func['name'])
    for pi, param in enumerate(func['params']):
        p_name = '_' + param['name']
        f.write('request.set%s(p%d);' % (p_name, pi))
    f.write('        request.send().wait(waitScope);\n')
    _emit_passthru(f, func)

def _emit_logger(f, func):
    _emit_log(f, func)
    _emit_passthru(f, func)

def main():
    with open('purple-protos/decls.json') as f:
        decls = json.load(f)
    config = StubConfig.load('stubs.yaml')
    with open('stubs.rs', 'w+') as f:
        f.write('use purpleproxy::{Proxy, with_proxy};\n')
        f.write('use purple;\n\n')
        for func in sorted(decls['functions'], key=lambda k: k['name']):
            if func['variadic'] or config.skip(func['name']):
                if not config.skip(func['name']):
                    print "WARNING: Skipping variadic %s" % (func['name'],)
                continue

            f.write('#[no_mangle]\n');
            f.write('pub fn %s(%s) -> %s {\n' % (func['name'], _to_arglist(func['params']), _to_rusttyp(func['return'])))
            f.write('    with_proxy(|proxy| {\n');
            if config.proxy(func['name']):
                _emit_proxy(f, func)
            else:
                _emit_logger(f, func)
            f.write('    })\n');
            f.write('}\n\n')

if __name__ == '__main__':
    main()
