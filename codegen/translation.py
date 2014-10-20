import sys
from collections import namedtuple

ParamDecl = namedtuple('ParamDecl', ['p_type', 'c_type', 'p_name', 'c_name'])

class Function(object):
    def __init__(self):
        self.p_name = None
        self.c_name = None
        self.variadic = False
        self.params = []
        self.ret = []

class Translator(object):
    def __init__(self, config):
        self._config = config

    @staticmethod
    def _mangle_name(name):
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

    def _mangle_param_type(self, func, param, type_):
        result = self._config.get_typeinfo(func.c_name, param)
        if result:
            return result
        sys.stderr.write('WARNING: Could not translate type %s\n' % (type_))
        return type_

    def _mangle_return_type(self, func, type_):
        return self._mangle_param_type(func, 'return', type_)

    def process_decls(self, decls):
        result = []
        for in_func in decls['functions']:
            func = Function()
            func.c_name = in_func['name']
            func.p_name = self._mangle_name(func.c_name)
            func.variadic = in_func['variadic']
            for pi, param in enumerate(in_func['params']):
                if not param['name']:
                    param['name'] = 'p%d' % pi
                p_type = self._mangle_param_type(func, param['name'], param['type'])
                p_name = self._mangle_name(param['name'])
                func.params.append(ParamDecl(c_type=param['type'],
                                             c_name=param['name'],
                                             p_type=p_type,
                                             p_name=p_name))
            if in_func['return'] != 'void':
                ret_p_type = self._mangle_return_type(func, in_func['return'])
                func.ret.append(ParamDecl(c_type=in_func['return'],
                                          c_name=None,
                                          p_type=ret_p_type,
                                          p_name='return'))
            result.append(func)
        return result

