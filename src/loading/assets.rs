use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::abilities::AbilityDefRaw;
use crate::player::player_def::PlayerDef;
use crate::stats::loader::{CalculatorDefRaw, StatDefRaw};

#[derive(Asset, TypePath)]
pub struct StatsConfigAsset {
    pub stat_ids: Vec<StatDefRaw>,
    pub calculators: Vec<CalculatorDefRaw>,
}

#[derive(Asset, TypePath)]
pub struct PlayerDefAsset(pub PlayerDef);

#[derive(Asset, TypePath)]
pub struct AbilityDefAsset(pub AbilityDefRaw);

#[derive(Default, TypePath)]
pub struct StatsConfigLoader;

#[derive(Default, TypePath)]
pub struct PlayerDefLoader;

#[derive(Default, TypePath)]
pub struct AbilityDefLoader;

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
}

impl AssetLoader for PlayerDefLoader {
    type Asset = PlayerDefAsset;
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
        let player_def: PlayerDef = ron::from_str(content)?;
        Ok(PlayerDefAsset(player_def))
    }

    fn extensions(&self) -> &[&str] {
        &["player.ron"]
    }
}

impl AssetLoader for AbilityDefLoader {
    type Asset = AbilityDefAsset;
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
        let ability_def: AbilityDefRaw = ron::from_str(content)?;
        Ok(AbilityDefAsset(ability_def))
    }

    fn extensions(&self) -> &[&str] {
        &["ability.ron"]
    }
}
