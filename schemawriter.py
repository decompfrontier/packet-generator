from datetime import datetime
from gen import Generator
from schema import *
import os

class SchemaParser:
    @staticmethod
    def _parse_enum(enum: Enum) -> GeneratorStruct:
        """Creates an intermediate rappresentation of the Python enumerator.

        :param enum: Python enumerator to parse
        :return: Synthetized generator data
        """
        ir = GeneratorStruct()
        ir.name = enum.__name__
        ir.class_type = ClassType.Enumerator

        if hasattr(ir, "__doc__"): # documentation of the enumerator
            ir.__doc__ = getattr(enum, "__doc__")
        
        __doc_fields__ = {}
        if hasattr(ir, "__doc_fields__"):
            # map that contains the documentation of all fields, we have to do this way
            # because python doesn't expose __doc__ for Enum fields
            __doc_fields__ = getattr(ir, "__doc_fields__")

        for name in enum._member_names_:
            if name == "__doc__" or name == "__doc_fields__":
                continue

            val = getattr(enum, name) # get all fields of the enum

            f = GeneratorField()
            f.name = name
            f.type_id = val

            if type(val.value) == str: ## this is a string enum
                ir.class_type = ClassType.EnumeratorString

            if val in __doc_fields__:
                f.__doc__ = __doc_fields__[val]

            ir.fields.append(f)
        
        return ir

    @staticmethod
    def parse(obj: type) -> GeneratorStruct:
        """Creates an intermediate rapresentation of a Python object.

        :param obj: Python class type to generate
        :return: Synthetized generator data
        """
        if issubclass(obj, Enum) or issubclass(obj, Flag):
            return SchemaParser._parse_enum(obj) # parse the enumerator if the type is compatible with it
        
        ir = GeneratorStruct()
        obj_inst = obj() # instance of the object type
        
        ir.name = obj.__name__
        if hasattr(obj, "__ir_key_group__"): # if there is ir_key_group then the type is a KeyJson
            ir.key = getattr(obj, "__ir_key_group__")

        ir.array_step = getattr(obj, "__ir_array_step__")    

        if hasattr(obj_inst, "__doc__"):
            ir.__doc__ = getattr(obj_inst, "__doc__")

        for field_name in dir(obj_inst): # as the object is a dataclass, when we instance it we can iterate for all the created fields
            if callable(getattr(obj_inst, field_name)) or field_name.startswith("__"): # skip all internal fields that starts with "__" and are not gettable
                continue
            
            # get the actual dictionary of the field
            field = getattr(obj_inst, field_name)

            if not type(field) == dict: # skip all fields that are not generated as dictionaries (this is enforced as it's how the format works)
                raise Exception("Invalid field declaration {}".format(field_name))
            
            ir_field = GeneratorField()
            ir_field.name = field_name

            # iterate the content of the dictionary
            for k, v in field.items():
                match k:
                    # read special keys
                    case "default":
                        ir_field.default_action = v
                    case "quote":
                        ir_field.quoted = True
                    case "string":
                        ir_field.force_as_string = True
                    case "doc":
                        ir_field.__doc__ = v
                    case _: # by default, a declaration of a field for the JSON mapping is "json name": type of this field
                        ir_field.key = k
                        ir_field.type_id = v

                        if v == None:
                            raise Exception("Referenced type of {} is broken".format(field_name))

            ir.fields.append(ir_field)
        
        return ir

class SchemaWriter:
    @staticmethod
    def write(py_file: str, output_dir: str, types: list[type], gen: Generator):
        """Generates the output file from packet specifications.

        :param py_file: Python file name
        :param output_dir: Output directory of the generated files
        :param types: List of Python types to be serialized from the python file
        :param gen: Generator of this type
        """

        types.sort(key=lambda m: m.__ir_declaration_line__) # sort the types by their declaration line

        def expand_output_dir(pyfile: str, outdir: str, ext: str) -> str:
            """Converts the Python import name (like data.net.userinfo) to a file name valid for the generator imports

            TODO: This function is hardcoded to support ONLY .net. and .mst. as directories, this should be changed.

            :param pyfile: Python file name (like data.net.userinfo)
            :param outdir: Output directory of the generated files
            :param ext: Generator extension (like .hpp)
            """
            to_subst = pyfile.rfind(".mst.")
            dir_root = ""
            sub_file = pyfile
            if to_subst != -1:
                dir_root = "mst_"
                sub_file = pyfile[to_subst + 5:]
            else:
                to_subst = pyfile.rfind(".net.")
                if to_subst != -1:
                    dir_root = "net_"
                    sub_file = pyfile[to_subst + 5:]
            
            sub_file = sub_file.replace(".", "_")
            outdir = os.path.abspath(outdir).replace("\\", "/")

            return "".join((outdir, "/", dir_root, sub_file, ext))

        pyfile_fix = py_file[:-3][2:].replace("/", ".") # converted python file name to an import name
        out_file = expand_output_dir(pyfile_fix, output_dir, gen.get_extension()) # output file for the generator

        # add a mark to the beginning of the file
        buffer = gen.get_start_mark(py_file) # buffer is a string that contains all the generated file content

        # list of modules that should be excluded when iterating the types declarated by this module
        exclude_mods = [
            "schema",
            "builtins",
            "datetime",
            pyfile_fix
        ]

        import_mods: list[inspect.ModuleType] = []
        structures: list[GeneratorStruct] = []

        # parse all types declarated in this module
        for type in types:
            ir = SchemaParser.parse(type)

            # inspect all the generated IRs to find if the generated file should
            # import other file types
            for field in ir.fields:
                mod = inspect.getmodule(field.type_id)
                if not mod.__name__ in exclude_mods: # skip default type mappings
                    if not mod in import_mods:
                        import_mods.append(mod)

            structures.append(ir)

        # generate the lines that declares all the extra modules that should be imported
        ## this is done for referencing a type outside of it's declaration (which is done in the schemas as normal Python imports)
        for mod in import_mods:
            mod_out = expand_output_dir(mod.__name__, output_dir, gen.get_extension())
            buffer = "".join((buffer, gen.add_import(mod_out)))

        # generate all the processed structures
        for ir in structures:
            buffer = "".join((buffer, gen.step(ir) ))
        
        # add a mark to the end of file
        buffer = "".join((buffer, gen.get_end_mark()))

        # finally write our file!
        with open(out_file, "wb") as fp:
            fp.write(buffer.encode("utf-8"))
