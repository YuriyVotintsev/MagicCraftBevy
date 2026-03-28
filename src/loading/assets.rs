use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::balance::GameBalance;
use crate::blueprints::BlueprintDefRaw;
use crate::expr::calc::CalcTemplateRaw;
use crate::skill_tree::types::SkillTreeDefRaw;
use crate::stats::display::StatDisplayRuleRaw;
use crate::stats::loader::StatDefRaw;

#[derive(Asset, TypePath)]
pub struct StatsConfigAsset {
    pub stat_ids: Vec<StatDefRaw>,
    #[allow(dead_code)]
    pub calcs: Vec<CalcTemplateRaw>,
    pub display: Vec<StatDisplayRuleRaw>,
}

#[derive(Asset, TypePath)]
pub struct BlueprintDefAsset(pub BlueprintDefRaw);

#[derive(Asset, TypePath)]
pub struct SkillTreeDefAsset(pub SkillTreeDefRaw);

#[derive(Asset, TypePath)]
pub struct GameBalanceAsset(pub GameBalance);

#[derive(Default, TypePath)]
pub struct SkillTreeDefLoader;

#[derive(Default, TypePath)]
pub struct GameBalanceLoader;

#[derive(Default, TypePath)]
pub struct StatsConfigLoader;

#[derive(Default, TypePath)]
pub struct BlueprintDefLoader;

impl AssetLoader for GameBalanceLoader {
    type Asset = GameBalanceAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let balance: GameBalance = ron::from_str(content)?;
        Ok(GameBalanceAsset(balance))
    }

    fn extensions(&self) -> &[&str] {
        &["balance.ron"]
    }
}

impl AssetLoader for StatsConfigLoader {
    type Asset = StatsConfigAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let asset: StatsConfigRaw = ron::from_str(content)?;
        Ok(StatsConfigAsset {
            stat_ids: asset.stat_ids,
            calcs: asset.calcs,
            display: asset.display,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["stats.ron"]
    }
}

#[derive(serde::Deserialize)]
struct StatsConfigRaw {
    stat_ids: Vec<StatDefRaw>,
    #[serde(default)]
    calcs: Vec<CalcTemplateRaw>,
    #[serde(default)]
    display: Vec<StatDisplayRuleRaw>,
}

impl AssetLoader for BlueprintDefLoader {
    type Asset = BlueprintDefAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let blueprint_def: BlueprintDefRaw = ron::from_str(content)?;
        Ok(BlueprintDefAsset(blueprint_def))
    }

    fn extensions(&self) -> &[&str] {
        &["ability.ron", "mob.ron", "hero.ron"]
    }
}

impl AssetLoader for SkillTreeDefLoader {
    type Asset = SkillTreeDefAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let def: SkillTreeDefRaw = ron::from_str(content)?;
        Ok(SkillTreeDefAsset(def))
    }

    fn extensions(&self) -> &[&str] {
        &["tree.ron"]
    }
}
