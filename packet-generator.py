from argparse import ArgumentParser
from schemawriter import SchemaWriter
from test import TesterFactory
from gen import GeneratorFactory
import os
import importlib
import inspect

def main():
    """Main function execution."""

    ap = ArgumentParser(
        prog="Packet generator",
        description="Packet generator for Brave Frontier data"
    )
    
    ap.add_argument("--test", action="store_true", required=False)
    ap.add_argument("-l", "--language", required=True)
    ap.add_argument("-s", "--schema", required=False)
    ap.add_argument("-o", "--outputdir", required=False)
    ap.add_argument("--exclude", required=False, nargs="+", action="extend")

    argz = ap.parse_args()

    if argz.test:
        # execute tester
        g = TesterFactory.get(argz.language)
        g.run()
    else:
        if not argz.schema:
            ap.print_usage()
            print("error: the following arguments are required: schema")
            return

        if not argz.outputdir:
            ap.print_usage()
            print("error: the following arguments are required: outputdir")
            return
        
        try:
            os.mkdir(argz.outputdir)
        except FileExistsError:
            pass
        
        gen = GeneratorFactory.get(argz.language)
        
        schema = argz.schema.lower()

        if argz.exclude == None:
            argz.exclude = []
        
        for root, _, files in os.walk("./data/"):
            root2 = root[7:]
            if "__pycache__" in root2 or root2 == "":
                continue

            dirroot = root.replace("./data/", "").replace("/", "_")

            files_to_import = []

            for file in files:
                if file[-3:] != ".py" or file[:2] == "__":
                    continue

                filename = file[:-3]
                pyfile = f"{root}/{file}"
                if schema == "*":
                    if not filename in argz.exclude:
                        files_to_import.append(pyfile)
                elif filename == schema:
                    files_to_import.append(pyfile)
            
            for pyfile in files_to_import:
                pyfile_fix = pyfile[:-3][2:].replace("/", ".")
                mod = importlib.import_module(pyfile_fix)
                add_types = []

                for name, obj in inspect.getmembers(mod, inspect.isclass):
                    if hasattr(obj, "__pkprocess__"):
                        add_types.append(obj)

                if len(add_types) < 1:
                    continue

                idx = pyfile_fix.rfind(".")
                outname = pyfile_fix
                if idx != -1:
                    outname = pyfile_fix[idx+1:]

                outname = "".join((argz.outputdir, "/", dirroot, "_", outname, gen.get_extension()))
                
                print(f"Writing module: {pyfile}")
                SchemaWriter.write(pyfile, outname, add_types, gen)

        

if __name__ == "__main__":
    main()
