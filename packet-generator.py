from argparse import ArgumentParser
from schemawriter import SchemaWriter
from gen import GeneratorFactory
import os
import importlib
import inspect

def main():
    """Main function of the packet generator."""

    ap = ArgumentParser(
        prog="Packet generator",
        description="Packet generator for Brave Frontier data"
    )
    
    ap.add_argument("-l", "--language", required=True)
    ap.add_argument("-s", "--schema", required=False)
    ap.add_argument("-o", "--outputdir", required=False)
    ap.add_argument("--exclude", required=False, nargs="+", action="extend")
    ap.add_argument("--debug", required=False, action="store_true", default=False)

    argz = ap.parse_args()

    if not argz.schema:
        ap.print_usage()
        print("error: the following arguments are required: schema")
        return

    if not argz.outputdir:
        ap.print_usage()
        print("error: the following arguments are required: outputdir")
        return
    
    # try to make the output dir just in case
    try:
        os.mkdir(argz.outputdir)
    except FileExistsError:
        pass
    
    # try to gets the generator for this, will raise an exception if failed
    gen = GeneratorFactory.get(argz.language)
    
    # gets the single schema to generate, or use "*" to include all the schemas inside the packet-generator directory
    schema_name = argz.schema.lower()

    if argz.exclude == None: # list of schemas to exclude during the generation
        argz.exclude = []

    # TODO: we should get the directory of "packet-generator.py" and use it as a base for data
    # NOTE: everything is hardcoded to have schemas only in data/mst and data/net
    
    schema_root_directory = "./data/"
    for root, _, files in os.walk(schema_root_directory): # iterate all files found in the data directory
        schema_root = root[len(schema_root_directory):]
        if "__pycache__" in schema_root or schema_root == "": # skip empty directores or the __pycache__ directory
            continue

        directory_root_in_python_module_name = root.replace(schema_root_directory, "").replace("/", ".")
        files_to_import = []

        for file in files: # iterate all files found in the subdirectories
            if file[-3:] != ".py" or file[:2] == "__": # skip all files that starts with "__" or files that aren't python ones
                continue

            filename = file[:-3]
            python_file = f"{root}/{file}"
            python_module_name = f"{directory_root_in_python_module_name}.{filename}"

            if argz.debug:
                print("Check: {} == {}?".format(schema_name, python_module_name))

            if schema_name == "*": # flag to indicate to load all schemas
                if not filename in argz.exclude: # skip the schemas that are manually excluded
                    files_to_import.append(python_file)
            elif python_module_name == schema_name:
                files_to_import.append(python_file)

        for file in files_to_import: # iterate all files that are flagged to be processed
            # import the python module
            file_in_python_module_name = file[:-3][2:].replace("/", ".")
            mod = importlib.import_module(file_in_python_module_name)
            add_types = []

            for _, obj in inspect.getmembers(mod, inspect.isclass): # get all declared members of the module that are classes
                # process only the classes that were declared in this module and are flagged as processable by their attributes
                if hasattr(obj, "__ir_declaration_line__") and obj.__module__ == file_in_python_module_name:
                    add_types.append(obj)

            if len(add_types) < 1:
                continue

            print(f"Writing module: {file}")
            SchemaWriter.write(file, argz.outputdir, add_types, gen)

if __name__ == "__main__":
    main()
