use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::affixes::{AffixDefRaw, OrbDefRaw};
use crate::artifacts::ArtifactDefRaw;
use crate::blueprints::BlueprintDefRaw;
use crate::player::hero_class::HeroClassRaw;
use crate::stats::display::StatDisplayRuleRaw;
use crate::stats::loader::{CalculatorDefRaw, StatDefRaw};

#[derive(Asset, TypePath)]
pub struct StatsConfigAsset {
    pub stat_ids: Vec<StatDefRaw>,
    pub calculators: Vec<CalculatorDefRaw>,
    pub display: Vec<StatDisplayRuleRaw>,
}

#[derive(Asset, TypePath)]
pub struct BlueprintDefAsset(pub BlueprintDefRaw);

#[derive(Asset, TypePath)]
pub struct ArtifactDefAsset(pub ArtifactDefRaw);

#[derive(Asset, TypePath)]
pub struct HeroClassAsset(pub HeroClassRaw);

#[derive(Asset, TypePath)]
pub struct AffixPoolAsset(pub Vec<AffixDefRaw>);

#[derive(Asset, TypePath)]
pub struct OrbConfigAsset(pub Vec<OrbDefRaw>);

#[derive(Default, TypePath)]
pub struct StatsConfigLoader;

#[derive(Default, TypePath)]
pub struct BlueprintDefLoader;

#[derive(Default, TypePath)]
pub struct ArtifactDefLoader;

#[derive(Default, TypePath)]
pub struct HeroClassLoader;

#[derive(Default, TypePath)]
pub struct AffixPoolLoader;

#[derive(Default, TypePath)]
pub struct OrbConfigLoader;

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
            calculators: asset.calculators,
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
    calculators: Vec<CalculatorDefRaw>,
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

impl AssetLoader for ArtifactDefLoader {
    type Asset = ArtifactDefAsset;
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
        let def: ArtifactDefRaw = ron::from_str(content)?;
        Ok(ArtifactDefAsset(def))
    }

    fn extensions(&self) -> &[&str] {
        &["artifact.ron"]
    }
}

impl AssetLoader for HeroClassLoader {
    type Asset = HeroClassAsset;
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
        let def: HeroClassRaw = ron::from_str(content)?;
        Ok(HeroClassAsset(def))
    }

    fn extensions(&self) -> &[&str] {
        &["class.ron"]
    }
}

impl AssetLoader for AffixPoolLoader {
    type Asset = AffixPoolAsset;
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
        let defs: Vec<AffixDefRaw> = ron::from_str(content)?;
        Ok(AffixPoolAsset(defs))
    }

    fn extensions(&self) -> &[&str] {
        &["affixes.ron"]
    }
}

impl AssetLoader for OrbConfigLoader {
    type Asset = OrbConfigAsset;
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
        let defs: Vec<OrbDefRaw> = ron::from_str(content)?;
        Ok(OrbConfigAsset(defs))
    }

    fn extensions(&self) -> &[&str] {
        &["orbs.ron"]
    }
}
