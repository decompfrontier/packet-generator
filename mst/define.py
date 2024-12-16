from schema import *

@keyjson(key_group = "VkoZ5t3K", single = True)
@configurable(filename = "define.json")
class DefineMst:
    max_zel = { "DXm2W8vY": long }
    max_karma = { "jSxdlwqd": long }
    max_friend_points = { "4YCzox9t": long }
    max_team_lv = { "Kt8H4LN7": int }
    max_arena_rank = { "tzCfGX83": int }
    max_unit_count = { "ouXxIY63": int }
    max_warehouse_count = { "5pjoGBC4": int }
    default_unit_count = { "k0xrd38b": int } # number of free unit slots when creating an account
    default_warehouse_count = { "8U93zxsf": int } # number of free warehouse slot when creating an account
    # TODO: what's this ?
    friendpoint_rein_friend = { "2oD1fmuX": int }
    friendpoint_rein_friend_rate = { "PnE6xo93": float }
    friendpoint_rein_friend_my = { "86sx4FfJ": int }
    friendpoint_rein_friend_my_rate = { "JBR0Po3b": float }
    friendpoint_rein_other = { "K10QSeuj": int }
    friendpoint_rein_other_rate = { "6p4YW7oc": float }
    friendpoint_rein_other_my = { "VkA3nu0b": int }
    friendpoint_rein_other_my_rate = { "mI1jW0X7": float }
    permit_invitation = { "02IgM6ib": int }
    ext_plus_max_count = { "4t3qX2kT": int }

    action_point_heal_count = { "C8KkHGa7": int } # energy
    fight_point_heal_count = { "sy9G24Su": int } # arena points
    unit_box_ext_count = { "I73XkAQi": int } # TODO: eh? number of gems?
    item_box_ext_count = { "CA01vo2Q": int }  # TODO: eh?
    continue_dia_count = { "QW3HiNv8": int } # TODO: number of continues?
    initial_unit = { "21ovwqYT": str } # TODO: I'm sure this is not JUST a string
    tutorial_dungeon_id = { "91nRcYWT": int }

    # TODO: what's this?
    recover_time_action = { "Ieq49JDy": int }
    recover_time_fight = { "0BPn68DG": int }
    arena_battle_time_limit = { "YR4HI56k": int }
    arena_need_mission_id = { "yFRYDj67": int } # mission to unlock the arena
    max_party_deck_count = { "WHy3BSm9": int }
    arena_tutorial_npc_info = { "6W4PdoJY": str } # TODO: I'm sure this is not JUST a string
    verify_flag = { "6GXx4LgZ": int }
    unit_mix_great_exp_rate = { "2inP0tCg": float }
    unit_mix_super_exp_rate = { "zn65EXYF": float }
    recover_time_frohun = { "3xAsgHL8": int }
    recover_time_raid = { "zkD98Hfy": int }
    raid_max_p = { "S92Hcor3": int }
    raid_bag_count = { "09EbcDmX": int }
    friend_ext_count = { "3Ep5akHJ": int }
    medal_max_num = { "xq0fSrw3": int } # which one?
    compaign_flag = { "MFz8YRS6": str }
    # note: are this the conversion rates?
    max_achieve_point = { "1JFcDr05": int }
    zel_per_achieve_point = { "SAb3m9wo": int }
    karma_per_achieve_point = { "KCG5f1AN": int }
    max_achieve_point_zel_per_day = { "4ARtfF7x": int }
    max_achieve_point_karma_per_day = { "K0sUIn8R": int }
    max_achieve_point_item_per_day = { "M1AJuFU6": int }
    max_achievement_challenge_accept_count = { "p3pXbuHA": int }
    max_achievement_record_challenge_accept_count = { "kmxPgJu9": int }
    tutorial_gatcha_id = { "uALQnngx": int }
    parse_overdrive_param = { "QylsZTpE": str } # this is NOT just a string
    colosseum_shop_ticket = { "OjAiNSoh": int }
    max_colosseum_ticket = { "924iwrJ9": int }
    max_cbp = { "woxAcRoH": int } # ??
    reset_fe_skill_dia_count = { "5csFoG1G": int }
    max_blacklist_count = { "DYMUxgt8": int }
    init_summoner_arm_id = { "deYOowYJ": int } # bf chapter 3?
    max_summoner_sp = { "QhV0G2zu": long }
    max_summoner_friend_point = { "W9bwut7Q": long }
    max_multi_p = { "6id2v7eN": int }
    dbb_crystal_values = { "jFdW1ipx": str } # this is NOT just a string
    dbb_fixed_settings_value = { "7o6lcc66": str }  # this is NOT just a string
    action_point_recover_fixed = { "hAiXsSPF": int, "omit_on_default": True }
    action_point_threshold = { "eRQvzLeF": int, "omit_on_default": True }
