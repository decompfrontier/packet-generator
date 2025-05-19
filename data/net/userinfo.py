from .gumi_live import GumiLiveInfo
from schema import *

'''
@keyjson(key_group = "Bnc4LpM8", array = False)
class UserAchievementInfo:
    pass

@keyjson(key_group = "8jBJ7uKR")
class UserArenaInfo:
    pass

@keyjson(key_group = "6C0kzwM5")
class UserBraveMedalInfo:
    pass

@keyjson(key_group = "UT1SVg59")
class UserClearMissionInfo:
    pass

@keyjson(key_group = "eFU7Qtb0")
class UserDungeonKeyInfo:
    pass

@keyjson(key_group = "nAligJSQ")
class UserEquipBoostItemInfo:
    pass

@keyjson(key_group = "71U5wzhI")
class UserEquipItemInfo:
    pass

@keyjson(key_group = "3kcmQy7B")
class UserFavorite:
    pass

@keyjson(key_group = "30uygM9m")
class UserGiftInfo:
    pass

@keyjson(key_group = "bd5Rj6pN")
class UserItemDictionaryInfo:
    pass
'''

@keyjson(key_group = "IKqx1Cn9", single = True)
class UserInfo:
    "Main object that holds all the player information."

    user_id = { "h7eY3sAK": str, "doc": "ID of the user for this session." }
    handle_name = { "B5JQyV8j": str, "doc": "Username shown in-game." }
    account_id = { "LZ2PfNm4": str, "doc": "Account ID used in account transfering." }
    password = { "4WSu1irc": str, "doc": "Password used for account transfering." }
    friend_id = { "98WfKiyA": str, "doc": "Player ID that is used for the friend system." }
    contact_id = { "90LWtVUN": str, "doc": "ID of the player used in the ticketing system for player support." }
    tutorial_status = { "9sQM2XcN": int, "doc": "Get the current tutorial script to execute." }
    tutorial_end_flag = { "sv6BEI8X": strbool, "doc": "Checks if the player has finished the tutorial or not."  }
    user_scenario_info = { "N4XVE1uA": str } # TODO: this is UserScenarioInfoList we just dont have this yet
    user_special_scenario_info = { "9yVsu21R": str } # TODO: this is UserSpecialScenarioInfoList we just dont have this yet
    model_change_count = { "nrg19RGe": int, "doc": "How many times has the user transferred to a new device. May be used to avoid getting first time playing rewards twice." }
    code_expire_date = { "iyJH5k6p": int }
    friend_invitation_flag = { "y2v7Sd01": int }
    early_bird_end = { "iN7cYU9i": long, "doc": "Time in seconds until the Gem banner sale ends." }
    debug_mode = { "5MPcr0sp": strbool, "doc": "Enables several debug features (like skipping summoning animation) on the account." }
    encrypt_iv = { "8kN1tgYU": str }
    encrypted_friend_id = { "PA0QwZs1": str }
    first_desc = { "7oV00FeR": str }
    dlc_url = { "23t3D28i": str, "default": DefaultType.Omit }
    feature_gate = { "a37D29iJ": str }
    unk = { "32k0ahkD": str } # v["32k0ahkD"] = "773c9af44721a014c7ed"; // TODO: discover what's this
    service_request_endpoint_param = { "ABh7acL2": str, "default": DefaultType.Omit, "doc": "Extra parameters that are passed to the DLS API server in the request object EXTRA_PARAMS." }
    # GUMI LIVE API V1/V2
    gumi_live_userid = { "iN7buP2h": str, "doc": "Gumi live API User ID. (used for example in IAP)" }
    gumi_live_token = { "iN7buP1i": str, "doc": "User token of the Gumi live API." }
    facebook_id = { "K29dp2Q": long, "doc": "Facebook ID of the associated account." }
    associated_user_id = { "uJP4aeg9": str }

@keyjson(key_group = "dX7S2Lc1")
class UserPartyDeckInfo:
    deck_type = { "U9ABSYEp": int }
    deck_num = { "zsiAn9P1": int }
    user_unit_id = { "edy7fq3L": int }
    member_type = { "gr48vsdJ": int }
    disp_order = { "XuJL4pc5": int }

'''
@keyjson(key_group = "4W6EhXLS", array = False)
class UserPurchaseInfo:
    pass

@keyjson(key_group = "Dp0MjKAf")
class UserReleaseInfo:
    pass

@keyjson(key_group = "d98mjNDc")
class UserSoundInfo:
    pass

@keyjson(key_group = "dhMmbm5p")
class UserSUmmonerArmsInfo:
    pass

@keyjson(key_group = "n5mdIUqj")
class UserSummonerInfo:
    pass

@keyjson(key_group = "zI2tJB7R")
class UserTeamArchive:
    pass

@keyjson(key_group = "PQ56vbkI")
class UserTeamArenaArchive:
    pass

@keyjson(key_group = "YRgx49WG")
class UserTownFacilityInfo:
    pass

@keyjson(key_group = "s8TCo2MS")
class UserTownLocationDetail:
    pass

@keyjson(key_group = "yj46Q2xw")
class UserTownLocationInfo:
    pass

@keyjson(key_group = "sxorQ3Mb")
class UserUnitDbbInfo:
    pass

@keyjson(key_group = "tR4katob")
class UserUnitDbbLevelInfo:
    pass

@keyjson(key_group = "GV81ctzR")
class UserUnitDictionary:
    pass

@keyjson(key_group = "9wjrh74P")
class UserWarehouseInfo:
    pass
'''

@json(array = False)
class UserTeamInfo:
    user_id = { "h7eY3sAK": str }
    level = { "D9wXQI2V": int }
    exp = { "d96tuT2E": long }
    max_action_point = { "YnM14RIP" : int }
    action_point = { "0P9X1YHs" : int }
    max_fight_point = { "9m5FWR8q" : int }
    fight_point = { "YS2JG9no" : int }
    max_unit_count = { "ouXxIY63" : int }
    add_unit_count = { "Px1X7fcd" : int }
    deck_cost = { "QYP4kId9" : int }
    max_equip_slot_count = { "gEX30r1b" : int }
    max_friend_count = { "3u41PhR2" : int }
    add_friend_count = { "2rR5s6wn" : int }
    friend_point = { "J3stQ7jd": int }
    zel = { "Najhr8m6" : long }
    karma = { "HTVh8a65" : long }
    brave_coin = { "03UGMHxF" : int }
    friend_message = { "bM7RLu5K" : str }
    warehouse_count = { "5pjoGBC4" : int }
    add_warehouse_count = { "iI7Wj6pM" : int }
    want_gift = { "s2WnRw9N" : str }
    present_count = { "EfinBo65" : int }
    friend_agree_count = { "qVBx7g2c" : int }
    gift_receive_count = { "1RQT92uE": int }
    action_rest_timer = { "f0IY4nj8" : int }
    fight_rest_timer = { "jp9s8IyY" : int }
    free_gems = { "92uj7oXB" : int }
    active_deck = { "Z0Y4RoD7" : int }
    summon_ticket = { "9r3aLmaB" : int }
    slot_game_flag = { "s3uU4Lgb" : int }
    rainbow_coin = { "KAZmxkgy" : int }
    brave_points_total = { "bya9a67k" : int }
    colosseum_ticket = { "lKuj3Ier" : int }
    arena_deck_num = { "gKNfIZiA" : int }
    reinforcement_deck = { "TwqMChon" : str } # TODO: this is reinforcement_deck + reinforcement_deck_ex1 + reinforcement_deck_ex2
    paid_gems = { "d37CaiX1" : int }
    mysterybox_count = { "Qo9doUsp" : int }
    completed_task_count = { "3a8b9D8i" : int }
    inbox_message_count = { "7qncTHUJ" : int }
    current_brave_points = { "22rqpZTo" : int }

@keyjson(key_group = "4ceMWH6k")
class UserUnitInfo:
    user_id = { "h7eY3sAK" : str }
    user_unit_id = { "edy7fq3L" : int }
    unit_id = { "pn16CNah" : int }
    unit_type_id = { "nBTx56W9" : int }
    unit_lv = { "D9wXQI2V" : int }
    exp = { "d96tuT2E" : int }
    total_exp = { "gQInj3H6" : int }
    base_hp = { "e7DK0FQT" : int }
    add_hp = { "cuIWp89g" : int }
    ext_hp = { "TokWs1B3" : int }
    limit_over_hp = { "ISj9u5VL" : int }
    base_atk = { "67CApcti" : int }
    add_atk = { "RT4CtH5d" : int }
    ext_atk = { "t4m1RH6Y" : int }
    limit_over_atk = { "D6bKH5eV" : int }
    base_def = { "q08xLEsy" : int }
    add_def = { "GcMD0hy6" : int }
    ext_def = { "e6mY8Z0k" : int }
    limit_over_def = { "3CsiQA0h" : int }
    base_heal = { "PWXu25cg" : int }
    add_heal = { "C1HZr3pb" : int }
    ext_heal = { "X6jf8DUw" : int }
    limit_over_heal = { "XJs2rPx0" : int }
    element = { "iNy0ZU5M" : str }
    leader_skill_id = {"oS3kTZ2W"  : int }
    skill_id = { "nj9Lw7mV" : int }
    skill_lv = { "3NbeC8AB" : int }
    extra_skill_id = { "iEFZ6H19" : int }
    extra_skill_lv = { "RQ5GnFE2": int }
    receive_date = { "Bvkx8s6M" : int }
    ext_count = { "5gXxT7LZ" : int }
    equipitem_frame_id = { "0R3qTPK9" : int }
    equipitem_id = { "Ge8Yo32T" : int }
    equipitem_frame_id2 = { "RXfC31FA" : int }
    equipitem_id2 = { "mZA7fH2v" : int }
    new_flag = { "dJNpLc81" : int }
    extra_passive_skill_id = { "cP83zNsv" : int }
    extra_passive_skill_id2 = { "LjY4DfRg": int }
    add_extra_passive_skill_id = { "T4rewHa9" : int }
    unit_img_type = { "2pAyFjmZ": int }
    fe_bp = { "bFQbZh3x": int }
    fe_used_bp = { "3RgneFpP" : int }
    fe_max_usable_bp = { "GIO9DTif" : int }
    fe_skill_info = { "Fnxab5CN" : str } # TODO: is this really a str?
    omni_level = { "49sa3sld" : int }

## --- request

@keyjson(key_group = "IKqx1Cn9", array = False)
class UserInfoReq(GumiLiveInfo):
    user_id = { "h7eY3sAK": str }
    contact_id = { "90LWtVUN": str }
    model_change_count = { "nrg19RGe": int }
    device_name = { "iN7buP0j": str }
    target_os = { "DFY3k6qp": str }
    build_platform_id = { "j2lk52Be": str }
    device_id = { "Ma5GnU0H": str }
    pointer_name = { "fKSzGDFb": str }
    first_desc_mst_req = { "7oV00FeR": str }
    notice_mst_list_req = { "aXf114Oz": str }
    minfo = { "236dItKo": str }

@keyjson(key_group = "KeC10fuL")
class MstUrlList:
    id = { "moWQ30GH": str }
    version = { "d2RFtP8T": str }

    