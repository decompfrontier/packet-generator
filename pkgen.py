from gen import *
from schema import *

def make_generator_type_from_enum(clz: Enum) -> GeneratorData:
    """Creates a generator data class (rapresentation of a JSON in the generator) from a Python enumerator.

    :param clz: Python enumerator to generate
    :return: Synthetized generator data
    """
    g = GeneratorData()
    g.name = clz.__name__
    g.class_type = ClassType.Enumerator

    if hasattr(g, "doc"):
        g.doc = getattr(g, "doc")
    
    doc_f = {}
    if hasattr(g, "doc_f"):
        doc_f = getattr(g, "doc_f")

    for name in clz._member_names_:
        if name == "doc" or name == "doc_f":
            continue

        val = getattr(clz, name)

        f = GeneratorField()
        f.name = name
        f.type_id = val

        if val in doc_f:
            f.doc = doc_f[val]

        g.fields.append(f)
    
    return g

def make_generator_type_from_schema(clz: type) -> GeneratorData:
    """Creates the generator data class (rapresentation of a JSON in the generator) from a Python type specification.

    :param clz: Python class to generate
    :return: Synthetized generator data
    """
    if issubclass(clz, Enum):
        return make_generator_type_from_enum(clz)
    
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

        if f == "doc":
            g.doc = getattr(q, f)
            continue
        elif not type(f_attr) == dict:
                    raise Exception("Invalid field {}".format(f))
        
        f_gen = GeneratorField()
        f_gen.name = f

        # this has to be done this way beause I can't know the key, maybe the format should be changed
        #  to have key: value ?        
        for k, v in f_attr.items():
            match k:
                case "omit_on_default":
                    f_gen.default_action = DefaultType.Omit
                case "empty_on_default":
                    f_gen.default_action = DefaultType.Empty
                case "quoted":
                    f_gen.quoted = True
                case "write_as_string":
                    f_gen.force_as_string = True
                case "doc":
                    f_gen.doc = v
                case _:
                    f_gen.key = k
                    f_gen.type_id = v

        g.fields.append(f_gen)
    
    return g

def output_file(py_file: str, out_file: str, lang: str, types: list[type]):
    """Generates the output file from packet specifications.

    :param pyfile: Python file name
    :param out_file: Output file path
    :param lang: Generator type
    :param types: List of types to be serialized from the python file
    """
    gen = GeneratorFactory.get(lang)

    buffer = gen.get_start_mark(datetime.now(), py_file)

    # generate all types!
    for x in types:
        q = make_generator_type_from_schema(x)
        buffer = "".join((buffer, gen.step(q) ))
    
    buffer = "".join((buffer, gen.get_end_mark()))

    with open(out_file, "wb") as fp:
        fp.write(buffer.encode("utf-8"))

def generate_data_for_gtest_glaze():
    """Test generator for C++ Google test glaze runtime."""

    from net.signalkey import SignalKey
    from net.challenge_arena import ChallengeArenaUserInfo
    from net.daily_login import DailyLoginRewardsUserInfo
    from net.logincampaign import UserLoginCampaignInfo
    from net.gme import GmeErrorFlags, GmeErrorCommand, GmeBody, GmeError, GmeHeader, GmeAction
    from mst.gatcha import GachaMst, GachaCategory
    from mst.npc import NpcMst
    from mst.logincampaign import LoginCampaignMst
    from mst.town import TownFacilityLvMst

    output_file("signalkey.py", "test/glazecpp/generated/signalkey.hpp", "c++", [ SignalKey ])
    output_file("challenge_arena.py", "test/glazecpp/generated/challenge_arena.hpp", "c++", [ ChallengeArenaUserInfo ])
    output_file("daily_login.py", "test/glazecpp/generated/daily_login.hpp", "c++", [ DailyLoginRewardsUserInfo ])
    output_file("logincampaign.py", "test/glazecpp/generated/logincampaign.hpp", "c++", [ UserLoginCampaignInfo, LoginCampaignMst ])
    output_file("gme.py", "test/glazecpp/generated/gme.hpp", "c++", [ GmeErrorFlags, GmeErrorCommand, GmeBody, GmeError, GmeHeader, GmeAction ])
    output_file("npc.py", "test/glazecpp/generated/npc.hpp", "c++", [ NpcMst ])
    output_file("gacha.py", "test/glazecpp/generated/gacha.hpp", "c++", [ GachaMst, GachaCategory ])
    output_file("town.py", "test/glazecpp/generated/town.hpp", "c++", [ TownFacilityLvMst ])

# TODO: Replace this with actual logic!
generate_data_for_gtest_glaze()
