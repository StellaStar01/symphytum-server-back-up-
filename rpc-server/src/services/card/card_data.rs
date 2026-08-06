use database::Database;
use resource::master::MasterTable;
use sqlx::Error;
use types::entity::master::{
    Card as CardMaster, CardLevel, CardLevelLimit, CardPotential, SkillTreeConnectEffect,
    SkillTreeEffect, SkillTreeNode,
};
use types::enums::{CardPotentialEffectType, SkillTreeEffectType};

/// current level: highest CardLevel[group] whose exp threshold is met, capped by the limit-break level limit.
pub fn level_of(card: &CardMaster, exp: i64, limit_break_count: i32) -> i32 {
    let mut level = 1;
    for l in CardLevel::table()
        .iter()
        .filter(|l| l.group_id == card.card_level_group_id)
    {
        if l.level > 1 && l.exp > 0 && l.exp <= exp {
            level = level.max(l.level);
        }
    }
    let limit = CardLevelLimit::table()
        .iter()
        .filter(|l| {
            l.group_id == card.card_level_limit_group_id && l.limit_break_count == limit_break_count
        })
        .map(|l| l.level_limit)
        .max()
        .unwrap_or(i32::MAX);
    level.min(limit)
}

pub fn parameter_base(card: &CardMaster, level: i32) -> i64 {
    CardLevel::table()
        .iter()
        .find(|l| l.group_id == card.card_level_group_id && l.level == level)
        .map(|l| l.parameter_base_value)
        .unwrap_or(0)
}

/// (all-parameter-up permil, connect effect level) from potentials at or below the upgrade count.
pub fn potential_bonus(card: &CardMaster, upgrade_count: i32) -> (i64, i32) {
    let mut bonus = 0i64;
    let mut connect = 1i32;
    for p in CardPotential::table()
        .iter()
        .filter(|p| p.group_id == card.card_potential_group_id && p.upgrade_count <= upgrade_count)
    {
        match CardPotentialEffectType::try_from(p.effect_type).unwrap_or_default() {
            CardPotentialEffectType::AllParameterUpPermilUp => bonus += p.value,
            CardPotentialEffectType::SkillTreeConnectEffectLevelUp => {
                connect = connect.max(p.value as i32);
            }
            _ => {}
        }
    }
    (bonus, connect)
}

/// ceil(permil * parameter/1000 * (1 + bonusPermil/1000)), mirrors the client.
pub fn attribute_value(parameter: i64, permil_multiply: i32, bonus_permil: i64) -> i64 {
    let v =
        permil_multiply as f32 * (parameter as f32 / 1000.0) * (1.0 + bonus_permil as f32 / 1000.0);
    v.ceil() as i64
}

/// flat + permil bonuses from released skill tree nodes, plus the connect-effect permil.
#[allow(clippy::too_many_arguments)]
pub fn skill_tree_bonus(
    card: &CardMaster,
    released_groups: &[String],
    connect_level: i32,
) -> (i64, i64, i64, i64, i64, i64) {
    let mut perf = 0i64;
    let mut tech = 0i64;
    let mut sense = 0i64;
    let mut perf_p = 0i64;
    let mut tech_p = 0i64;
    let mut sense_p = 0i64;
    for group in released_groups {
        for node in SkillTreeNode::table()
            .iter()
            .filter(|n| n.group_id == *group)
        {
            if node.skill_tree_effect_id.is_empty() {
                continue;
            }
            let Some(eff) = SkillTreeEffect::table()
                .iter()
                .find(|e| e.id == node.skill_tree_effect_id)
            else {
                continue;
            };
            let v = eff.value;
            match SkillTreeEffectType::try_from(eff.effect_type).unwrap_or_default() {
                SkillTreeEffectType::PerformanceUp => perf += v,
                SkillTreeEffectType::TechniqueUp => tech += v,
                SkillTreeEffectType::SenseUp => sense += v,
                SkillTreeEffectType::PerformanceUpPermilUp => perf_p += v,
                SkillTreeEffectType::TechniqueUpPermilUp => tech_p += v,
                SkillTreeEffectType::SenseUpPermilUp => sense_p += v,
                SkillTreeEffectType::AllParameterUp => {
                    perf += v;
                    tech += v;
                    sense += v;
                }
                SkillTreeEffectType::AllParameterUpPermilUp => {
                    perf_p += v;
                    tech_p += v;
                    sense_p += v;
                }
                _ => {}
            }
        }
    }
    let connect_permil = SkillTreeConnectEffect::table()
        .iter()
        .filter(|c| c.id == card.skill_tree_connect_effect_id && c.level <= connect_level)
        .max_by_key(|c| c.level)
        .map(|c| c.effect_permil_up)
        .unwrap_or(0);
    (
        perf,
        tech,
        sense,
        perf_p + connect_permil,
        tech_p + connect_permil,
        sense_p + connect_permil,
    )
}

/// full parameter block for one owned card.
pub async fn parameters_for(
    db: &Database,
    uid: &str,
    card_id: &str,
) -> Result<Option<(i64, i64, i64, i64)>, Error> {
    let Some(card) = CardMaster::table().iter().find(|c| c.id == card_id) else {
        return Ok(None);
    };
    let Some(uc) = db.user_card(uid, card_id).await? else {
        return Ok(None);
    };
    let level = level_of(&card, uc.exp, uc.level_limit_break_count);
    let parameter = parameter_base(&card, level);
    let (bonus, _) = potential_bonus(&card, uc.potential_upgrade_count);
    let performance = attribute_value(parameter, card.performance_permil_multiply, bonus);
    let technique = attribute_value(parameter, card.technique_permil_multiply, bonus);
    let sense = attribute_value(parameter, card.sense_permil_multiply, bonus);
    Ok(Some((parameter, performance, technique, sense)))
}
