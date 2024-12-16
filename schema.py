
# --- custom decorators

'''
    Serializable JSON attribute
'''
def json(cls = None, /, *, array = True, single = False):
    def _process_json(cls, array):
        cls.is_array = array
        cls.is_single = single
        
        return cls
    
    def wrap(cls):
        return _process_json(cls, array)
    
    if cls is None:
        return wrap
    
    return wrap(cls)

'''
    Serializable JSON with a base group
'''
def keyjson(cls: object = None, /, *, key_group: str, array = True, single = False):
    def _wrap(cls: object):
        cls.key_group = key_group
        return json(cls, array = array, single = single)
    
    if cls is None:
        return _wrap
    
    return _wrap(cls)

'''
    A configurable file
'''
def configurable(cls: object = None, /, *, filename: str):
    def _process_cfg(cls: object, filename: str) -> object:
        cls.configure_name = filename
        return cls
    
    def wrap(cls: object):
        return _process_cfg(cls, filename)
    
    if cls is None:
        return wrap
    
    return wrap(cls)

# --- custom types, only useful for names

class strbool(type):
    pass

class long(type):
    pass

class intbool(type):
    pass

class datetimeunix(type):
    pass
