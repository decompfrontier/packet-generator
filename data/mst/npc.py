from schema import *

@json
class NpcPartyInfo:
    id = { "P_USER_UNIT_ID": int }
    type = { "P_MEMBER_TYPE": int }
    disp_order = { "P_DISPORDER": int }

@json
class NpcTeamInfo:
    user_id = { "P_USER_ID": str }
    lv = { "P_LV": int }
    friend_message = { "P_FRIEND_MESSAGE": str }

@json
class NpcUnitInfo:
    id = { "P_UNIT_ID": int }
    party_id = { "P_USER_UNIT_ID": int }
    type = { "P_UNIT_TYPE_ID": int }
    lv = { "P_LV": int }
    hp = { "P_HP": int }
    atk = { "P_ATK": int }
    deff = { "P_DEF": int }
    hel = { "P_HEL": int }
    skill_id = { "P_SKILL_LV": int }
    skill_lv = { "P_SKILL_ID": int }
    equip_item_id = { "P_EQP_ITEM_ID": int }

@keyjson(key_group = "hV5vWu6C", array=ArrayStep.Array)
class NpcMst:
    id = { "7zyHb5h9": int }
    handle_name = { "B5JQyV8j": str }
    arena_rank_id = { "JmFn3g9t": int }
    team = { "g94bDiaS": NpcTeamInfo, "string": True }
    parties = { "oPsmRC18": NpcPartyInfo, "string": True }
    units = { "bS9s4GCp": NpcUnitInfo, "string": True }
