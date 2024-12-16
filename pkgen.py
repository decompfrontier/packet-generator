from gen import *
from schema import *

def make_generator_type_from_enum(clz: Enum) -> GeneratorData:
    g = GeneratorData()
    g.name = clz.__name__
    g.class_type = ClassType.Enumerator

    for name in clz._member_names_:
        val = getattr(clz, name)
        
        f = GeneratorField()
        f.name = name
        f.type_id = val
        g.fields.append(f)
    
    return g

def make_generator_type_from_schema(clz: type) -> GeneratorData:
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
        if not type(f_attr) == dict:
                    raise Exception("Invalid field {}".format(f))
        
        f_gen = GeneratorField()
        f_gen.name = f
        
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
                case _:
                    f_gen.key = k
                    f_gen.type_id = v

        g.fields.append(f_gen)
    
    return g

def output_file(py_file: str, out_file: str, lang: str, types: list[type]):
    gen = GeneratorFactory.get(lang)

    buffer = gen.get_start_mark(datetime.now(), py_file)

    for x in types:
        q = make_generator_type_from_schema(x)
        buffer = "".join((buffer, gen.step(q) ))
    
    buffer = "".join((buffer, gen.get_end_mark()))

    with open(out_file, "wb") as fp:
        fp.write(buffer.encode("utf-8"))

def generate_data_for_gtest_glaze():
    from net.signalkey import SignalKey
    from net.challenge_arena import ChallengeArenaUserInfo
    from net.daily_login import DailyLoginRewardsUserInfo
    from net.logincampaign import UserLoginCampaignInfo
    from net.gme import GmeErrorID, GmeErrorOperation, GmeBody, GmeError, GmeHeader, GmeAction
    from mst.gatcha import GachaMst
    from mst.npc import NpcMst
    from mst.logincampaign import LoginCampaignMst
    from mst.town import TownFacilityLvMst

    output_file("signalkey.py", "test/glazecpp/generated/signalkey.hpp", "c++", [ SignalKey ])
    output_file("challenge_arena.py", "test/glazecpp/generated/challenge_arena.hpp", "c++", [ ChallengeArenaUserInfo ])
    output_file("daily_login.py", "test/glazecpp/generated/daily_login.hpp", "c++", [ DailyLoginRewardsUserInfo ])
    output_file("logincampaign.py", "test/glazecpp/generated/logincampaign.hpp", "c++", [ UserLoginCampaignInfo, LoginCampaignMst ])
    output_file("gme.py", "test/glazecpp/generated/gme.hpp", "c++", [ GmeErrorID, GmeErrorOperation, GmeBody, GmeError, GmeHeader, GmeAction ])
    output_file("npc.py", "test/glazecpp/generated/npc.hpp", "c++", [ NpcMst ])
    output_file("gacha.py", "test/glazecpp/generated/gacha.hpp", "c++", [ GachaMst ])
    output_file("town.py", "test/glazecpp/generated/town.hpp", "c++", [ TownFacilityLvMst ])

generate_data_for_gtest_glaze()
