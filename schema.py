"""This module provides custom decorators that are used for serialization of the data."""

def json(cls = None, /, *, array = True, single = False):
    """This decorator rapresents a JSON fields or class.
    
    For example:
        {
            ...
        }

    :param cls: Base object to serialize
    :param array: True if the object should be generated as a JSON array
    :param single: True if the object is an array that contains only one element
    """

    def _process_json(cls, array):
        cls.is_array = array
        cls.is_single = single
        
        return cls
    
    def wrap(cls):
        return _process_json(cls, array)
    
    if cls is None:
        return wrap
    
    return wrap(cls)

def keyjson(cls: object = None, /, *, key_group: str, array = True, single = False):
    """This decorator represents a JSON class that contains a custom key group.
    
    For example:
        {
            "key_group": [
                ...
            ]
        }

    :param cls: Base object to serialize
    :param key_group: Key group name (like 6FrKacq7)
    :param array: True if the object should be generated as a JSON array
    :param single: True if the object is an array that contains only one element
    """

    def _wrap(cls: object):
        cls.key_group = key_group
        return json(cls, array = array, single = single)
    
    if cls is None:
        return _wrap
    
    return _wrap(cls)

def configurable(cls: object = None, /, *, filename: str):
    """This decorator represents a class that should contain stubs for loading it from a JSON file.
    
    For example: the Gatcha configuration that should be loaded from the server configuration.
    
    :param cls: Base object to serialize
    :param filename: Name of the configuration file to use for reading"""
    def _process_cfg(cls: object, filename: str) -> object:
        cls.configure_name = filename
        return cls
    
    def wrap(cls: object):
        return _process_cfg(cls, filename)
    
    if cls is None:
        return wrap
    
    return wrap(cls)

# custom types, only useful for names

class strbool(type):
    """A boolean that is serialized as a string. ("0" or "1" defined as normal boolean for generators)"""
    pass

class long(type):
    """A 64-bit integer."""
    pass

class intbool(type):
    """A boolean that is serialized as an integer. (0 or 1 defined as normal boolean for generators)"""
    pass

class datetimeunix(type):
    """A date time stored in unix epoch. (long with time like 1740763959)"""
    pass
