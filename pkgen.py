from argparse import ArgumentParser
from schemawriter import SchemaWriter
from test import TesterFactory

def main():
    ap = ArgumentParser(
        prog="Packet generator",
        description="Packet generator for Brave Frontier data"
    )
    
    ap.add_argument("--test", action="store_true", required=False)
    ap.add_argument("-l", "--language", required=True)
    ap.add_argument("-s", "--schema", required=False)
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
        
        if argz.schema == "*":
            pass

        

if __name__ == "__main__":
    main()
