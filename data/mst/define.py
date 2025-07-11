from schema import *

@keyjson(key_group = "VkoZ5t3K", array=ArrayStep.Single)
class DefineMst:
    max_zel = { "DXm2W8vY": longstr }
    max_karma = { "jSxdlwqd": longstr }
    max_friend_points = { "4YCzox9t": longstr }
    max_team_lv = { "Kt8H4LN7": intstr }
    max_arena_rank = { "tzCfGX83": intstr }
    max_unit_count = { "ouXxIY63": intstr }
    max_warehouse_count = { "5pjoGBC4": intstr }
    default_unit_count = { "k0xrd38b": intstr } # number of free unit slots when creating an account
    default_warehouse_count = { "8U93zxsf": intstr } # number of free warehouse slot when creating an account
    # TODO: what's this ?
    friendpoint_rein_friend = { "2oD1fmuX": intstr }
    friendpoint_rein_friend_rate = { "PnE6xo93": floatstr }
    friendpoint_rein_friend_my = { "86sx4FfJ": intstr }
    friendpoint_rein_friend_my_rate = { "JBR0Po3b": floatstr }
    friendpoint_rein_other = { "K10QSeuj": intstr }
    friendpoint_rein_other_rate = { "6p4YW7oc": floatstr }
    friendpoint_rein_other_my = { "VkA3nu0b": intstr }
    friendpoint_rein_other_my_rate = { "mI1jW0X7": floatstr }
    permit_invitation = { "02IgM6ib": intstr }
    ext_plus_max_count = { "4t3qX2kT": intstr }

    action_point_heal_count = { "C8KkHGa7": intstr } # energy
    fight_point_heal_count = { "sy9G24Su": intstr } # arena points
    unit_box_ext_count = { "I73XkAQi": intstr } # TODO: eh? number of gems?
    item_box_ext_count = { "CA01vo2Q": intstr }  # TODO: eh?
    continue_dia_count = { "QW3HiNv8": intstr } # TODO: number of continues?
    initial_unit = { "21ovwqYT": str } # TODO: I'm sure this is not JUST a string
    tutorial_dungeon_id = { "91nRcYWT": intstr }

    # TODO: what's this?
    recover_time_action = { "Ieq49JDy": intstr }
    recover_time_fight = { "0BPn68DG": intstr }
    arena_battle_time_limit = { "YR4HI56k": intstr }
    arena_need_mission_id = { "yFRYDj67": intstr } # mission to unlock the arena
    max_party_deck_count = { "WHy3BSm9": intstr }
    arena_tutorial_npc_info = { "6W4PdoJY": str } # TODO: I'm sure this is not JUST a string
    verify_flag = { "6GXx4LgZ": intstr }
    unit_mix_great_exp_rate = { "2inP0tCg": floatstr }
    unit_mix_super_exp_rate = { "zn65EXYF": floatstr }
    recover_time_frohun = { "3xAsgHL8": intstr }
    recover_time_raid = { "zkD98Hfy": intstr }
    raid_max_p = { "S92Hcor3": intstr }
    raid_bag_count = { "09EbcDmX": intstr }
    friend_ext_count = { "3Ep5akHJ": intstr }
    medal_max_num = { "xq0fSrw3": intstr } # which one?
    compaign_flag = { "MFz8YRS6": str }
    # note: are this the conversion rates?
    max_achieve_point = { "1JFcDr05": intstr }
    zel_per_achieve_point = { "SAb3m9wo": intstr }
    karma_per_achieve_point = { "KCG5f1AN": intstr }
    max_achieve_point_zel_per_day = { "4ARtfF7x": intstr }
    max_achieve_point_karma_per_day = { "K0sUIn8R": intstr }
    max_achieve_point_item_per_day = { "M1AJuFU6": intstr }
    max_achievement_challenge_accept_count = { "p3pXbuHA": intstr }
    max_achievement_record_challenge_accept_count = { "kmxPgJu9": intstr }
    tutorial_gatcha_id = { "uALQnngx": intstr }
    parse_overdrive_param = { "QylsZTpE": str } # this is NOT just a string
    colosseum_shop_ticket = { "OjAiNSoh": intstr }
    max_colosseum_ticket = { "924iwrJ9": intstr }
    max_cbp = { "woxAcRoH": intstr } # ??
    reset_fe_skill_dia_count = { "5csFoG1G": intstr }
    max_blacklist_count = { "DYMUxgt8": intstr }
    init_summoner_arm_id = { "deYOowYJ": commalist[int] } # bf chapter 3?
    max_summoner_sp = { "QhV0G2zu": longstr }
    max_summoner_friend_point = { "W9bwut7Q": longstr }
    max_multi_p = { "6id2v7eN": intstr }
    dbb_crystal_values = { "jFdW1ipx": str } # this is NOT just a string
    dbb_fixed_settings_value = { "7o6lcc66": str }  # this is NOT just a string
    action_point_recover_fixed = { "hAiXsSPF": int, "default": DefaultType.Omit }
    action_point_threshold = { "eRQvzLeF": int, "default": DefaultType.Omit }
