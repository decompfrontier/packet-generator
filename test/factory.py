
from .tester import Tester
from .glazecpp import GlazeCppTester

class TesterFactory:
    """Factory pattern for testers."""
    @staticmethod
    def get(name: str) -> Tester:
        """Gets the generator based from the specified mapping."""
        GENERATOR_MAPPING :dict[str, type] = {
            "c++":          GlazeCppTester
        }        
        if not name in GENERATOR_MAPPING:
            raise Exception("Invalid generator: {}".format(name))
        
        return GENERATOR_MAPPING[name]()
