import yaml

class StubConfig(object):
    def __init__(self, dct):
        self._dict = dct

    @classmethod
    def load(cls, fname):
        with open(fname, 'r') as f:
            return cls(yaml.safe_load(f.read()))

    def get_typeinfo(self, fname, param):
        return self._dict.get('typeinfo', {}).get(fname, {}).get(param)

    def proxy(self, name):
        return name in self._dict.get('proxy', [])

    def skip(self, name):
        return (name in self._dict.get('skip', []) or
                name in self._dict.get('passthrough', []))
