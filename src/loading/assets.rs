use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::actors::abilities::AbilitiesBalance;
use crate::actors::mobs::MobsBalance;
use crate::balance::GameBalance;

#[derive(Asset, TypePath)]
pub struct GameBalanceAsset(pub GameBalance);

#[derive(Asset, TypePath)]
pub struct MobsBalanceAsset(pub MobsBalance);

#[derive(Asset, TypePath)]
pub struct AbilitiesBalanceAsset(pub AbilitiesBalance);

#[derive(Default, TypePath)]
pub struct GameBalanceLoader;

#[derive(Default, TypePath)]
pub struct MobsBalanceLoader;

#[derive(Default, TypePath)]
pub struct AbilitiesBalanceLoader;

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

impl AssetLoader for MobsBalanceLoader {
    type Asset = MobsBalanceAsset;
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
        let balance: MobsBalance = ron::from_str(content)?;
        Ok(MobsBalanceAsset(balance))
    }

    fn extensions(&self) -> &[&str] {
        &["mobs.ron"]
    }
}

impl AssetLoader for AbilitiesBalanceLoader {
    type Asset = AbilitiesBalanceAsset;
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
        let balance: AbilitiesBalance = ron::from_str(content)?;
        Ok(AbilitiesBalanceAsset(balance))
    }

    fn extensions(&self) -> &[&str] {
        &["abilities.ron"]
    }
}
