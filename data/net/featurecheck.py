from schema import *

"""This module contains the configuration of the server and enabled contents."""

@json(array = False)
class FeatureCheck:
    """Features of the server"""

    randall = { "randall": intbool }
    frontier_hunter = { "frontierhunter": intbool, "doc": "Enables the frontier hunter" }

    # new
    dungeon_key = { "dungeon_key": intbool, "doc": "Enables the requirement to use keys for vortex dungeons" }
    exp_window = { "exp_window": intbool }
    trial = { "trial": intbool }
    # end of new

    send_banner_mst = { "banner_mst": intbool, "doc": "Enables sending the Banner MST" }
    
    # new
    reload_file_mst = { "reload_file_mst": intbool }
    recommend_app = { "recommend_app": intbool }
    # end of new

    raid = { "raid": intbool, "doc": "Enables the raids" }
    
    # new
    raid_beta = { "raid_beta": intbool, "doc": "Enables the beta testing mode of raids" }
    dict = { "dict": intbool }
    enable_character_voice = { "character_voice": intbool, "doc": "Enables the voices during attacks on units" }
    enable_raid_battle_restart = { "raid_battle_restart": intbool, "doc": "Allows restarting a raid battle" }
    # end of new

    enable_auto_battle = { "autobattle": intbool, "doc": "Enables auto button in battles" }

    # new
    multi_summon = { "multisummon": intbool, "doc": "Enables multisummoning" }
    # end of new

    multiaccept = { "multiaccept": intbool }
    facebook_stories = {"facebook_stories": intbool }
    enable_name_change = { "name_change_func": intbool, "doc": "Enables name changing" }

    # new
    randall_facility = { "randall_facility": intbool }
    dailytask_notify = { "dailytask_notify": intbool }
    dailylogin_gem = { "dailylogin_gem": intbool }
    # end of new

    shop_friend = { "shop_friend": intbool }
    slot = { "slot": intbool }
    sort = { "sort": intbool }
    dungeon_key_count_on_redeem = { "dungeon_key_cnt": int, "doc": "Sets the dungeon keys gave to the players everytime they redeem a key" } # 1

    # new
    dlc_popup_android = { "dlc_popup_android": intbool }
    select_dlc_android = { "select_dlc_android": intbool }
    #fps_low = { "fps_low": float } # 30.0
    # end of new

    battle_item_limit = { "battle_item_limit": int } # 500

    # new
    #bb_timer = { "bb_timer": int } # 4
    social_special = { "social_special": intbool }
    #end of new

    enable_cheats = { "ischeat_enable": intbool }

    # new
    arx_punish_enable = { "arx_punish_enable": intbool }
    # end of new

    arx_popup_enable = { "arx_popup_enable": intbool }
    arx_popup_ios_enable = { "arx_popup_ios_enable": intbool }

    # new
    randall_library_memories = { "randall_library_memories": intbool }
    full_unit_ills_esclude = { "full_unit_ills_esclude": intbool }
    # end of new

    check_for_corrupted_mst = { "corrupted_mst_check": intbool, "doc": "Let the cliet checks for corrupted MSTs" }
    tutorial_skip = { "tutorial_skip": intbool, "doc": "Skips the tutorial" }
    enable_grand_quests = { "bf_campaign_grand_quest": intbool, "doc": "Enables the Grand quest"}
    enable_achievements = { "bf_achievement": intbool, "doc": "Enables achievements in Randall capital" }
    bf_achievement_ext = { "bf_achievement_ext": intbool }

    # new
    force_using_summon_tickets = { "force_use_summon_tickets": intbool, "doc": "Forces the use of Summon tickets before Gems" }
    # end of new

    feature_gate = { "feature_gate": intbool }
    enable_challenge_arena = { "challenge_arena": intbool, "doc": "Enables the Challenge Arena" }
    challenge_arena_banner_lock = { "challenge_arena_banner_lock": intbool }
    video_ads = { "video_ads": intbool }
    enable_new_video_ads_slot = { "video_ads_slot": intbool, "doc": "Enables the slot machines with video ads" }
    enable_battle_speed_button = { "battle_speed": intbool, "doc": "Enables the battle speedup button" }
    enable_battle_speed_button_in_ca = { "battle_speed_ca": intbool, "doc": "Enables the battle speedup button in Challenge Arena" }
    enable_battle_speed_button_in_arena = { "battle_speed_arena_pvp": intbool, "doc": "Enables the battle speedup button in Arena" }
    autobattle_record = { "autobattle_record": intbool }
    enable_colosseum = { "colosseum_enable": intbool, "doc": "Enables the colosseum" }
    sandbag_enable = { "sandbag_enable": intbool }

    # new
    exclude_ca_fusion_unit = { "exclude_ca_fusion_unit": intbool }
    enable_resummons = { "sg_resummon_gacha_enable": intbool, "doc": "Enables the resummoning banner" }
    sg_target_bundle_flag = { "sg_target_bundle_flag": intbool }
    # end of new

    guild_visible = { "guild_visible": intbool }
    enable_guilds = { "guid": intbool, "doc": "Enables guids system"}

    # new
    enable_old_video_ads_slot = { "old_video_ads_slot": intbool }
    # end of new

    daily_dungeon_list = { "daily_dungeon_list": str } # TODO: this is a list with a comma!! (100000,100050,100100,100200,100300,100310,100320,100400,100500)
    new_164_trial = { "new_164_trial": intbool }
    enable_mystery_chests = { "mystery_chest": intbool, "doc": "Enables mystery chests" }
    freepaid_gems = { "freepaid_gems": intbool }
    cooldown_timer = { "cooldown_timer": int } # 5
    va_sp_skill = { "va_sp_skill": intbool }
    frontiergate_plus = { "frontiergate_plus": intbool }
    enable_summon_categories = { "gacha_category": intbool, "doc": "Enables category in summons" }
    unit_type_bonus_skill = { "unit_type_bonus_skill": intbool }

    # tags unused by version 2.9.16 but where still found in the answer
    fish = { "fish": intbool }
    felloplay_community = { "felloplay_community": intbool }
    supersonic_google = { "supersonic_google": intbool }
    google_felloplay = { "google_felloplay": intbool }
    felloplay_community_ios = { "felloplay_community_ios": intbool }
    amazon_coins_reward_control = { "amazon_conins_reward_control": intbool }
