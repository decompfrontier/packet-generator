from schema import *
from data.mst.versioninfo import *
from data.mst.logincampaign import *
from data.mst.userlevel import *
from data.mst.town import *
from data.mst.dungeonkey import *
from data.mst.arena import *
from data.mst.gatcha import *
from data.mst.define import *
from data.mst.npc import *
from data.mst.slots import *
from data.mst.banner import *
from data.mst.excludeddungeon import *
from data.mst.passiveskill import *
from data.mst.noticeinfo import *
from data.net.signalkey import *
from data.net.challenge_arena import *
from data.mst.dailytaskbonus import *
from data.net.guild import *
from data.net.daily_login import *
from data.net.summonerjournaluserinfo import *
from data.net.userinfo import *

@json
class InitializeResp:
    """Response of the initialize command"""
    
    # cachable
    loginCampagin = { "" : LoginCampaignMst }
    loginCampaignReward = { "" : LoginCampaignReward }
    progression = { "" : UserLevelMst, "doc": "Configuration of the user stats progression" }
    mst = { "" : VersionInfo, "doc": "Configuration of the versions of the MSTs" }
    townFacility = { "" : TownFacilityMst }
    townFacilityLv = { "" : TownFacilityLvMst }
    townLocation = { "" : TownLocationMst }
    townLocationLv = { "" : TownLocationLvMst }
    dungeonKeys = { "" : DungeonKeyMst, "doc": "Configuration of the Vortex dungeon with keys" }
    arenaRanks = { "" : ArenaRankMst, "doc": "PVP Arena rankings" }
    gachaEffects = { "": GachaEffectMst }
    gachas = { "": GachaMst, "doc": "Configuration of current Gacha banners" }
    defines = { "": DefineMst, "doc": "Configuration of the server" }
    npcs = { "": NpcMst }
    bannerInfo = { "": BannerInfoMst }
    extraPassiveSkills = { "": ExtraPassiveSkillMst }
    noticeInfo = { "": NoticeInfo, "doc": "Configuration of notices" }
    # user specific
    signalKey = { "": SignalKey }
    challengeArenaUserInfo = { "": ChallengeArenaUserInfo }
    dailyTaskBonuses = { "": DailyTaskBonusMst }
    dailyTaskPrizes = { "" : DailyTaskPrizeMst }
    dailyTasks = { "": DailyTaskMst }
    dailyLoginRewards = { "": DailyLoginRewardsUserInfo }
    guild = { "": GuildInfo, "doc": "Guild information" }
    videoAdSlots = { "": VideoAdsSlotGameInfo }
    summonerJournal = { "": SummonerJournalUserInfo }
    userInfo = { "": UserInfoResp, "doc": "User information" }
