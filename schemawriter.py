from datetime import datetime
from gen import Generator
from schema import *
import os

class SchemaParser:
    @staticmethod
    def _parse_enum(clz: Enum) -> GeneratorData:
        """Creates a generator data class (rapresentation of a JSON in the generator) from a Python enumerator.

        :param clz: Python enumerator to generate
        :return: Synthetized generator data
        """
        g = GeneratorData()
        g.name = clz.__name__
        g.class_type = ClassType.Enumerator

        if hasattr(g, "__doc__"):
            g.__doc__ = getattr(g, "__doc__")
        
        __doc_fields__ = {}
        if hasattr(g, "__doc_fields__"):
            __doc_fields__ = getattr(g, "__doc_fields__")

        for name in clz._member_names_:
            if name == "__doc__" or name == "__doc_fields__":
                continue

            val = getattr(clz, name)

            f = GeneratorField()
            f.name = name
            f.type_id = val

            if type(val.value) == str: ## this is a string enum
                g.class_type = ClassType.EnumeratorString

            if val in __doc_fields__:
                f.__doc__ = __doc_fields__[val]

            g.fields.append(f)
        
        return g

    @staticmethod
    def parse(clz: type) -> GeneratorData:
        """Creates the generator data class (rapresentation of a JSON in the generator) from a Python type specification.

        :param clz: Python class to generate
        :return: Synthetized generator data
        """
        if issubclass(clz, Enum) or issubclass(clz, Flag):
            return SchemaParser._parse_enum(clz)
        
        g = GeneratorData()
        q = clz()
        g.name = clz.__name__
        if hasattr(clz, "key_group"):
            g.key = getattr(clz, "key_group")
        if clz.is_single:
            g.array_step = ArrayStep.Single
        elif not clz.is_array:
            g.array_step = ArrayStep.NoArray
        
        for f in dir(q):
            if callable(getattr(q, f)) or f.startswith("__") or f == "is_single" or f == "key_group" or f == "is_array" or f == "configure_name":
                continue
            
            f_attr = getattr(q, f)

            if f == "__doc__":
                g.__doc__ = getattr(q, f)
                continue
            elif not type(f_attr) == dict:
                        raise Exception("Invalid field {}".format(f))
            
            f_gen = GeneratorField()
            f_gen.name = f

            # this has to be done this way beause I can't know the key, maybe the format should be changed
            #  to have key: value ?        
            for k, v in f_attr.items():
                match k:
                    case "default":
                        f_gen.default_action = v
                    case "quote":
                        f_gen.quoted = True
                    case "string":
                        f_gen.force_as_string = True
                    case "doc":
                        f_gen.__doc__ = v
                    case _:
                        f_gen.key = k
                        f_gen.type_id = v

                        if v == None:
                            raise Exception("Rferenced type is broken")

            g.fields.append(f_gen)
        
        return g

class SchemaWriter:
    @staticmethod
    def write(py_file: str, output_dir: str, types: list[type], gen: Generator):
        """Generates the output file from packet specifications.

        :param pyfile: Python file name
        :param out_file: Output file path
        :param types: List of types to be serialized from the python file
        :param gen: Generator type
        """

        types.sort(key=lambda m: m.__pkprocess__)

        def expand_output_dir(pyfile: str, outdir: str, ext: str) -> str:
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

        pyfile_fix = py_file[:-3][2:].replace("/", ".")
        out_file = expand_output_dir(pyfile_fix, output_dir, gen.get_extension())

        buffer = gen.get_start_mark(py_file)

        exclude_mods = [
            "schema",
            "builtins",
            "datetime",
            pyfile_fix
        ]

        import_mods: list[inspect.ModuleType] = []
        schemas: list[GeneratorData] = []

        # parse all files
        for x in types:
            q = SchemaParser.parse(x)

            # inspect all the schema to find it's imports
            for field in q.fields:
                mod = inspect.getmodule(field.type_id)
                if not mod.__name__ in exclude_mods:
                    if not mod in import_mods:
                        import_mods.append(mod)

            schemas.append(q)

        for mod in import_mods:
            mod_out = expand_output_dir(mod.__name__, output_dir, gen.get_extension())
            buffer = "".join((buffer, gen.add_import(mod_out)))

        for q in schemas:
            buffer = "".join((buffer, gen.step(q) ))
        
        buffer = "".join((buffer, gen.get_end_mark()))

        with open(out_file, "wb") as fp:
            fp.write(buffer.encode("utf-8"))
